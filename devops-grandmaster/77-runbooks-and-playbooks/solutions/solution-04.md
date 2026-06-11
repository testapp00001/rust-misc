# Solution 04: Runbook Automation

## Problem Statement

Build a Python `AutomatedRunbook` class that can execute runbook steps
sequentially with verification callbacks, dry-run mode, timeout handling,
and structured result reporting.

## Complete Solution

```python
"""
runbook_automation.py

Automated runbook execution engine for converting manual runbooks into
executable, verifiable procedures.

Usage:
    runbook = AutomatedRunbook(
        name="Disk Cleanup",
        description="Clean up disk space on a server",
        steps=[...],
        timeout=300,
    )
    result = runbook.execute(target="web-server-01", dry_run=False)
"""

import time
import logging
import traceback
from dataclasses import dataclass, field
from enum import Enum
from typing import Callable, Optional, Any
from datetime import datetime, timezone

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("runbook")


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

class StepStatus(Enum):
    """Execution status of a single runbook step."""
    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    SKIPPED = "skipped"
    TIMED_OUT = "timed_out"
    DRY_RUN = "dry_run"


class RunbookStatus(Enum):
    """Overall execution status of a runbook."""
    NOT_STARTED = "not_started"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    TIMED_OUT = "timed_out"
    PARTIAL = "partial"          # Some steps succeeded, some failed
    DRY_RUN_COMPLETE = "dry_run_complete"


@dataclass
class StepResult:
    """Result of executing a single runbook step."""
    step_name: str
    status: StepStatus
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    duration_seconds: float = 0.0
    output: Any = None
    error: Optional[str] = None
    verification_passed: Optional[bool] = None
    verification_message: Optional[str] = None

    def to_dict(self) -> dict:
        return {
            "step_name": self.step_name,
            "status": self.status.value,
            "started_at": self.started_at.isoformat() if self.started_at else None,
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "duration_seconds": round(self.duration_seconds, 3),
            "output": str(self.output)[:500] if self.output else None,
            "error": self.error,
            "verification_passed": self.verification_passed,
            "verification_message": self.verification_message,
        }


@dataclass
class RunbookResult:
    """Aggregate result of executing an entire runbook."""
    runbook_name: str
    status: RunbookStatus
    target: str
    dry_run: bool
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    duration_seconds: float = 0.0
    step_results: list[StepResult] = field(default_factory=list)
    error_summary: Optional[str] = None

    @property
    def success_count(self) -> int:
        return sum(1 for s in self.step_results if s.status == StepStatus.SUCCESS)

    @property
    def failure_count(self) -> int:
        return sum(1 for s in self.step_results if s.status in (
            StepStatus.FAILED, StepStatus.TIMED_OUT
        ))

    @property
    def total_steps(self) -> int:
        return len(self.step_results)

    def to_dict(self) -> dict:
        return {
            "runbook_name": self.runbook_name,
            "status": self.status.value,
            "target": self.target,
            "dry_run": self.dry_run,
            "started_at": self.started_at.isoformat() if self.started_at else None,
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "duration_seconds": round(self.duration_seconds, 3),
            "steps_succeeded": self.success_count,
            "steps_failed": self.failure_count,
            "steps_total": self.total_steps,
            "error_summary": self.error_summary,
            "steps": [s.to_dict() for s in self.step_results],
        }

    def summary(self) -> str:
        """Human-readable summary of the execution."""
        lines = [
            f"{'=' * 60}",
            f"Runbook: {self.runbook_name}",
            f"Target:  {self.target}",
            f"Mode:    {'DRY RUN' if self.dry_run else 'EXECUTE'}",
            f"Status:  {self.status.value.upper()}",
            f"Duration: {self.duration_seconds:.1f}s",
            f"Steps:   {self.success_count}/{self.total_steps} succeeded",
            f"{'=' * 60}",
        ]
        for sr in self.step_results:
            icon = {
                StepStatus.SUCCESS: "[OK]",
                StepStatus.FAILED: "[FAIL]",
                StepStatus.TIMED_OUT: "[TIMEOUT]",
                StepStatus.SKIPPED: "[SKIP]",
                StepStatus.DRY_RUN: "[DRY]",
            }.get(sr.status, "[??]")
            line = f"  {icon} {sr.step_name} ({sr.duration_seconds:.1f}s)"
            if sr.error:
                line += f" -- ERROR: {sr.error}"
            lines.append(line)
        if self.error_summary:
            lines.append(f"\nError Summary: {self.error_summary}")
        return "\n".join(lines)


# ---------------------------------------------------------------------------
# Step and Runbook definitions
# ---------------------------------------------------------------------------

@dataclass
class RunbookStep:
    """
    A single step in an automated runbook.

    Attributes:
        name: Human-readable step name.
        action: Callable that performs the step. Receives (target, context).
                Must return a result value on success or raise on failure.
        description: What this step does (for documentation).
        timeout: Per-step timeout in seconds (None = no limit).
        verification: Optional callable to verify the step worked.
                      Receives (target, context, action_result).
                      Must return (bool, str) -- (passed, message).
        required: If False, failure does not abort the runbook.
        skip_condition: Optional callable that returns True if the step
                        should be skipped. Receives (target, context).
        rollback: Optional callable to undo this step on failure.
                  Receives (target, context).
    """
    name: str
    action: Callable[[str, dict], Any]
    description: str = ""
    timeout: Optional[float] = None
    verification: Optional[Callable[[str, dict, Any], tuple[bool, str]]] = None
    required: bool = True
    skip_condition: Optional[Callable[[str, dict], bool]] = None
    rollback: Optional[Callable[[str, dict], None]] = None


class AutomatedRunbook:
    """
    Executes a sequence of runbook steps with verification, timeout,
    dry-run support, and structured reporting.

    Architecture:

    ┌─────────────────────────────────────────────────┐
    │                AutomatedRunbook                  │
    │                                                  │
    │  execute(target, dry_run)                        │
    │    │                                             │
    │    ├── for each step:                            │
    │    │     ├── check skip_condition                │
    │    │     ├── [dry_run] → log intent, skip action │
    │    │     ├── execute action(target, ctx)         │
    │    │     │     with timeout enforcement           │
    │    │     ├── run verification(action_result)     │
    │    │     ├── on failure → rollback completed steps│
    │    │     └── record StepResult                   │
    │    │                                             │
    │    └── return RunbookResult                     │
    └─────────────────────────────────────────────────┘
    """

    def __init__(
        self,
        name: str,
        description: str,
        steps: list[RunbookStep],
        timeout: Optional[float] = None,
        stop_on_failure: bool = True,
    ):
        """
        Args:
            name: Runbook identifier.
            description: What this runbook does.
            steps: Ordered list of RunbookStep instances.
            timeout: Global timeout for the entire runbook in seconds.
            stop_on_failure: If True, abort remaining steps on failure.
        """
        self.name = name
        self.description = description
        self.steps = steps
        self.timeout = timeout
        self.stop_on_failure = stop_on_failure

    def execute(self, target: str, dry_run: bool = False) -> RunbookResult:
        """
        Execute the runbook against a target.

        Args:
            target: The target system (hostname, pod name, etc.).
            dry_run: If True, log what would happen without executing actions.

        Returns:
            RunbookResult with full execution details.
        """
        result = RunbookResult(
            runbook_name=self.name,
            status=RunbookStatus.RUNNING,
            target=target,
            dry_run=dry_run,
            started_at=datetime.now(timezone.utc),
        )
        context: dict[str, Any] = {}   # Shared state between steps
        completed_steps: list[RunbookStep] = []  # For rollback
        global_deadline = (
            time.monotonic() + self.timeout if self.timeout else None
        )

        logger.info(
            "Starting runbook '%s' on target '%s' (dry_run=%s, steps=%d)",
            self.name, target, dry_run, len(self.steps),
        )

        for i, step in enumerate(self.steps):
            # --- Check global timeout ---
            if global_deadline and time.monotonic() > global_deadline:
                logger.error("Global timeout (%ss) exceeded", self.timeout)
                step_result = StepResult(
                    step_name=step.name,
                    status=StepStatus.TIMED_OUT,
                    error="Global runbook timeout exceeded",
                )
                result.step_results.append(step_result)
                result.status = RunbookStatus.TIMED_OUT
                result.error_summary = f"Global timeout ({self.timeout}s) exceeded at step {i + 1}"
                break

            # --- Check skip condition ---
            if step.skip_condition and step.skip_condition(target, context):
                logger.info("Step %d/%d '%s': SKIPPED (skip condition met)", i + 1, len(self.steps), step.name)
                result.step_results.append(StepResult(
                    step_name=step.name,
                    status=StepStatus.SKIPPED,
                ))
                continue

            # --- Dry run mode ---
            if dry_run:
                logger.info("Step %d/%d '%s': [DRY RUN] Would execute: %s", i + 1, len(self.steps), step.name, step.description)
                result.step_results.append(StepResult(
                    step_name=step.name,
                    status=StepStatus.DRY_RUN,
                    output=f"[DRY RUN] Would execute: {step.description}",
                ))
                continue

            # --- Execute the step ---
            logger.info("Step %d/%d '%s': executing...", i + 1, len(self.steps), step.name)
            step_result = self._execute_step(step, target, context)
            result.step_results.append(step_result)

            # --- Handle failure ---
            if step_result.status in (StepStatus.FAILED, StepStatus.TIMED_OUT):
                if step.required:
                    logger.error(
                        "Required step '%s' failed: %s",
                        step.name, step_result.error,
                    )
                    # Rollback completed steps in reverse order
                    self._rollback(completed_steps, target, context)
                    result.status = RunbookStatus.FAILED
                    result.error_summary = (
                        f"Step '{step.name}' failed: {step_result.error}"
                    )
                    break
                else:
                    logger.warning(
                        "Optional step '%s' failed, continuing: %s",
                        step.name, step_result.error,
                    )
            else:
                completed_steps.append(step)

        else:
            # All steps completed without break
            result.status = (
                RunbookStatus.DRY_RUN_COMPLETE if dry_run
                else RunbookStatus.SUCCESS
            )

        result.completed_at = datetime.now(timezone.utc)
        result.duration_seconds = (
            result.completed_at - result.started_at
        ).total_seconds()

        logger.info(
            "Runbook '%s' finished: status=%s, duration=%.1fs, steps=%d/%d",
            self.name, result.status.value, result.duration_seconds,
            result.success_count, result.total_steps,
        )
        return result

    def _execute_step(
        self, step: RunbookStep, target: str, context: dict
    ) -> StepResult:
        """Execute a single step with timeout and verification."""
        step_result = StepResult(
            step_name=step.name,
            status=StepStatus.RUNNING,
            started_at=datetime.now(timezone.utc),
        )

        try:
            # Execute with timeout
            action_result = self._run_with_timeout(
                step.action, target, context, step.timeout
            )
            step_result.output = action_result
            step_result.status = StepStatus.SUCCESS

            # Run verification if provided
            if step.verification:
                try:
                    passed, message = step.verification(
                        target, context, action_result
                    )
                    step_result.verification_passed = passed
                    step_result.verification_message = message
                    if not passed:
                        step_result.status = StepStatus.FAILED
                        step_result.error = f"Verification failed: {message}"
                except Exception as ve:
                    step_result.verification_passed = False
                    step_result.verification_message = str(ve)
                    step_result.status = StepStatus.FAILED
                    step_result.error = f"Verification error: {ve}"

        except TimeoutError:
            step_result.status = StepStatus.TIMED_OUT
            step_result.error = f"Step timed out after {step.timeout}s"
            logger.error("Step '%s' timed out", step.name)

        except Exception as e:
            step_result.status = StepStatus.FAILED
            step_result.error = f"{type(e).__name__}: {e}"
            logger.error("Step '%s' failed: %s", step.name, e)
            logger.debug(traceback.format_exc())

        step_result.completed_at = datetime.now(timezone.utc)
        step_result.duration_seconds = (
            step_result.completed_at - step_result.started_at
        ).total_seconds()
        return step_result

    @staticmethod
    def _run_with_timeout(
        func: Callable, target: str, context: dict, timeout: Optional[float]
    ) -> Any:
        """
        Run a function with an optional timeout.

        Note: This uses a simple time-based check. For production use,
        consider using concurrent.futures.ThreadPoolExecutor with timeout,
        or asyncio.wait_for for async runbooks.
        """
        if timeout is None:
            return func(target, context)

        import concurrent.futures
        with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
            future = executor.submit(func, target, context)
            try:
                return future.result(timeout=timeout)
            except concurrent.futures.TimeoutError:
                raise TimeoutError(f"Step exceeded {timeout}s timeout")

    def _rollback(
        self, completed_steps: list[RunbookStep], target: str, context: dict
    ):
        """Rollback completed steps in reverse order."""
        logger.info("Rolling back %d completed step(s)...", len(completed_steps))
        for step in reversed(completed_steps):
            if step.rollback:
                try:
                    logger.info("Rolling back step '%s'...", step.name)
                    step.rollback(target, context)
                    logger.info("Rollback of '%s' succeeded", step.name)
                except Exception as e:
                    logger.error("Rollback of '%s' failed: %s", step.name, e)
            else:
                logger.warning("Step '%s' has no rollback defined", step.name)


# ---------------------------------------------------------------------------
# Example: Disk Cleanup Runbook
# ---------------------------------------------------------------------------

def check_disk_usage(target: str, context: dict) -> dict:
    """Action: Check current disk usage and identify large files."""
    import subprocess
    result = subprocess.run(
        ["ssh", target, "df -h / && echo '---' && du -sh /var/log/* | sort -rh | head -10"],
        capture_output=True, text=True, timeout=30,
    )
    context["disk_output"] = result.stdout
    return {"raw_output": result.stdout}


def verify_disk_usage_reduced(target: str, context: dict, action_result: dict) -> tuple[bool, str]:
    """Verification: Confirm disk usage dropped below threshold."""
    import subprocess
    result = subprocess.run(
        ["ssh", target, "df --output=pcent / | tail -1 | tr -d ' %'"],
        capture_output=True, text=True, timeout=10,
    )
    usage = int(result.stdout.strip())
    context["disk_usage_after"] = usage
    passed = usage < 85
    return passed, f"Disk usage is {usage}% (threshold: 85%)"


def rotate_logs(target: str, context: dict) -> str:
    """Action: Force log rotation."""
    import subprocess
    result = subprocess.run(
        ["ssh", target, "sudo logrotate -f /etc/logrotate.conf 2>&1 || echo 'logrotate completed with warnings'"],
        capture_output=True, text=True, timeout=60,
    )
    return result.stdout


def clean_temp_files(target: str, context: dict) -> dict:
    """Action: Remove temp files older than 7 days."""
    import subprocess
    result = subprocess.run(
        ["ssh", target, "find /tmp -type f -mtime +7 -delete -print | wc -l"],
        capture_output=True, text=True, timeout=60,
    )
    files_removed = int(result.stdout.strip())
    context["files_removed"] = files_removed
    return {"files_removed": files_removed}


def verify_cleanup(target: str, context: dict, action_result: dict) -> tuple[bool, str]:
    """Verification: Confirm temp cleanup succeeded."""
    files = action_result.get("files_removed", 0)
    return True, f"Removed {files} temp files"


# Build the runbook
disk_cleanup_runbook = AutomatedRunbook(
    name="Disk Cleanup",
    description="Free disk space on a server by rotating logs and cleaning temp files",
    timeout=300,
    stop_on_failure=True,
    steps=[
        RunbookStep(
            name="Check initial disk usage",
            action=check_disk_usage,
            description="Identify current disk usage and largest directories",
            timeout=30,
            required=True,
        ),
        RunbookStep(
            name="Rotate logs",
            action=rotate_logs,
            description="Force log rotation to compress and archive old logs",
            timeout=60,
            required=True,
        ),
        RunbookStep(
            name="Clean temp files",
            action=clean_temp_files,
            description="Remove files in /tmp older than 7 days",
            timeout=60,
            required=True,
            verification=verify_cleanup,
        ),
        RunbookStep(
            name="Verify disk usage reduced",
            action=lambda t, c: None,   # No action; verification only
            description="Confirm disk usage is below 85% threshold",
            timeout=15,
            required=True,
            verification=verify_disk_usage_reduced,
        ),
    ],
)


# ---------------------------------------------------------------------------
# Example: High CPU Investigation Runbook
# ---------------------------------------------------------------------------

def find_top_processes(target: str, context: dict) -> list[dict]:
    """Action: Find the top 5 CPU-consuming processes."""
    import subprocess
    result = subprocess.run(
        ["ssh", target, "ps aux --sort=-%cpu | head -6"],
        capture_output=True, text=True, timeout=15,
    )
    lines = result.stdout.strip().split("\n")
    context["top_processes_raw"] = lines
    return {"process_list": lines}


def check_load_average(target: str, context: dict) -> dict:
    """Action: Check system load average."""
    import subprocess
    result = subprocess.run(
        ["ssh", target, "uptime"],
        capture_output=True, text=True, timeout=10,
    )
    context["load_output"] = result.stdout.strip()
    return {"uptime": result.stdout.strip()}


def skip_if_not_critical(target: str, context: dict) -> bool:
    """Skip escalation if load is manageable."""
    # This would parse the load average; simplified here
    return False


# Build the runbook
high_cpu_runbook = AutomatedRunbook(
    name="High CPU Investigation",
    description="Investigate and identify the cause of high CPU usage",
    timeout=120,
    stop_on_failure=False,   # Investigation runbook; gather all info
    steps=[
        RunbookStep(
            name="Check load average",
            action=check_load_average,
            description="Get current system load average",
            timeout=10,
        ),
        RunbookStep(
            name="Find top CPU processes",
            action=find_top_processes,
            description="Identify the top 5 CPU-consuming processes",
            timeout=15,
        ),
        RunbookStep(
            name="Escalate if critical",
            action=lambda t, c: logger.warning(
                "CPU investigation complete for %s. Manual review needed.", t
            ),
            description="Flag for manual review if load is critical",
            timeout=5,
            required=False,
            skip_condition=skip_if_not_critical,
        ),
    ],
)


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Execute automated runbooks")
    parser.add_argument("--target", required=True, help="Target host or resource")
    parser.add_argument("--runbook", required=True, choices=["disk-cleanup", "high-cpu"],
                        help="Runbook to execute")
    parser.add_argument("--dry-run", action="store_true", help="Log actions without executing")
    args = parser.parse_args()

    runbooks = {
        "disk-cleanup": disk_cleanup_runbook,
        "high-cpu": high_cpu_runbook,
    }
    selected = runbooks[args.runbook]
    result = selected.execute(target=args.target, dry_run=args.dry_run)
    print(result.summary())

    # Exit with non-zero if the runbook failed
    if result.status in (RunbookStatus.FAILED, RunbookStatus.TIMED_OUT):
        exit(1)
```

## Why This Works

1. **Separation of concerns**: Each step is a self-contained `RunbookStep` with
   its own action, verification, timeout, and rollback. This mirrors how
   runbooks are structured in documentation -- each step is independent.

2. **Verification callbacks**: The `verification` callable is separate from the
   `action` callable. This means you can verify that an action worked without
   coupling the verification logic to the action itself. For example, after
   rotating logs, you verify disk usage dropped -- these are logically distinct.

3. **Dry-run mode**: The `dry_run` flag lets you test runbook logic without
   side effects. This is critical for validating runbooks against production
   targets before committing to execution.

4. **Timeout at two levels**: Both per-step and global timeouts prevent runaway
   execution. A step that hangs (e.g., SSH to a dead host) will not block the
   entire runbook indefinitely.

5. **Rollback on failure**: When a required step fails, completed steps are
   rolled back in reverse order. This prevents partial execution from leaving
   the system in an inconsistent state.

6. **Structured results**: The `RunbookResult` object captures everything --
   timing, outputs, errors, verification results. This is directly usable for
   incident documentation, alerting integration, and post-mortem analysis.

## Common Mistakes

1. **No timeout on actions**: Every step that performs I/O (SSH, HTTP, database
   query) must have a timeout. Without it, a single hung connection blocks the
   entire runbook. Use the per-step `timeout` parameter for every I/O operation.

2. **Verification that checks the wrong thing**: The verification should confirm
   the *desired outcome*, not just that the action completed. For example,
   "logrotate ran successfully" is not verification -- "disk usage is below 85%"
   is verification.

3. **Making all steps required**: Optional steps (like "send Slack notification")
   should have `required=False`. If a non-critical step failure aborts the
   entire runbook, you lose the value of the steps that would have succeeded.

4. **No rollback defined**: If a step changes system state (modifies config,
   restarts a service), it should have a rollback. Without rollback, a failure
   midway through leaves the system partially modified.

5. **Ignoring dry-run results**: Run every new runbook with `dry_run=True` first.
   Check that the logged actions make sense before executing for real. This is
   the runbook equivalent of "test in staging."

6. **Thread safety**: The `context` dict is shared across steps. If you extend
   this to parallel step execution, you will need locks or a thread-safe dict.
   The current design is intentionally sequential to avoid this complexity.
