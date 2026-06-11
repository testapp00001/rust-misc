# Exercise 04: Runbook Automation

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Build a Python `AutomatedRunbook` class that can execute runbook steps programmatically, verify
results after each step, handle failures with rollback, and produce an execution log. This
exercise bridges the gap between human-readable runbooks and machine-executable automation.

## Scenario

Your team has 50+ runbooks, many of which follow the same pattern:
1. Run a command (shell, kubectl, psql, curl, etc.).
2. Verify the output matches an expected pattern.
3. If verification fails, either retry, run an alternative step, or roll back.
4. Proceed to the next step.
5. Produce a log of everything that happened.

You are building the engine that can execute these runbooks.

```
+-------------------+
|   Runbook YAML    |
| (human-readable)  |
+--------+----------+
         |
         v
+-------------------+     +-------------------+
| AutomatedRunbook  |---->| Execution Engine  |
|                   |     |                   |
| - load()          |     | - run_command()   |
| - execute()       |     | - verify_result() |
| - rollback()      |     | - retry()         |
| - get_log()       |     | - log_step()      |
+-------------------+     +-------------------+
         |
         v
+-------------------+
|  Execution Log    |
| (JSON report)     |
+-------------------+
```

## Tasks

### Part A: Define the Data Model

Create the data classes that represent a runbook and its steps. Use Python dataclasses or
Pydantic models.

```python
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional

class StepStatus(Enum):
    PENDING = "pending"
    RUNNING = "running"
    SUCCESS = "success"
    FAILED = "failed"
    SKIPPED = "skipped"
    ROLLED_BACK = "rolled_back"

class OnFailure(Enum):
    ABORT = "abort"           # Stop execution immediately
    RETRY = "retry"           # Retry the step
    CONTINUE = "continue"     # Skip and move to next step
    ROLLBACK = "rollback"     # Execute rollback steps

# TODO: Define these dataclasses
# @dataclass
# class Verification:
#     ...

# @dataclass
# class Step:
#     ...

# @dataclass
# class RollbackStep:
#     ...

# @dataclass
# class Runbook:
#     ...
```

Define:
- `Verification`: Contains the command to run, the expected output (regex or exact match), and a description.
- `Step`: Contains name, description, the command to execute, optional verification, on_failure behavior, max_retries, and an optional list of rollback steps.
- `RollbackStep`: Contains name and command to execute.
- `Runbook`: Contains name, description, version, author, and a list of Steps.

<details><summary>Hint</summary>
Each Step should have its own rollback steps because different steps may need different rollback
procedures. The `on_failure` field determines what happens when a step fails -- this is the
decision logic that makes the automation flexible.
</details>

### Part B: Implement the AutomatedRunbook Class

Implement the core class with these methods:

```python
class AutomatedRunbook:
    def __init__(self, runbook: Runbook, dry_run: bool = False):
        """
        Initialize the automated runbook.
        If dry_run is True, commands are logged but not executed.
        """
        pass

    def execute(self) -> bool:
        """
        Execute all steps in order. Returns True if all steps succeeded.
        Handles failures according to each step's on_failure policy.
        """
        pass

    def _run_step(self, step: Step) -> StepStatus:
        """
        Execute a single step: run command, verify result, handle failure.
        """
        pass

    def _run_command(self, command: str) -> tuple[int, str]:
        """
        Execute a shell command. Returns (return_code, output).
        """
        pass

    def _verify(self, verification: Verification, output: str) -> bool:
        """
        Verify command output against expected pattern.
        Supports exact match and regex.
        """
        pass

    def _retry_step(self, step: Step) -> StepStatus:
        """
        Retry a failed step up to max_retries times with exponential backoff.
        """
        pass

    def _rollback(self, executed_steps: list[Step]) -> None:
        """
        Execute rollback steps for all executed steps in reverse order.
        """
        pass

    def get_execution_log(self) -> list[dict]:
        """
        Return the full execution log as a list of dicts.
        """
        pass
```

Requirements:
- Use `subprocess.run()` with timeout for command execution.
- Implement exponential backoff for retries (1s, 2s, 4s, ...).
- Rollback must execute in reverse order of the steps that were executed.
- The execution log must capture: step name, command, output, status, duration, and any error messages.
- `dry_run` mode must log what *would* happen without executing anything.

<details><summary>Hint</summary>
Use `subprocess.run(command, shell=True, capture_output=True, text=True, timeout=30)`.
For exponential backoff: `time.sleep(2 ** attempt)`.
For rollback reversal: keep a list of executed steps, then iterate in reverse.
</details>

### Part C: Create a Sample Runbook YAML File

Write a YAML file that defines a runbook for "Restart a Failing Microservice":

```yaml
name: "Restart Failing Microservice"
description: "Automated restart of a service that is failing health checks"
version: "1.0"
author: "sre-team"
steps:
  - name: "check_service_status"
    description: "Check if the service is failing health checks"
    command: "kubectl get pods -l app=myservice -o jsonpath='{.items[*].status.phase}'"
    verification:
      description: "At least one pod should not be Running"
      expected: "^(?!.*Running.*Running.*Running).*$"
      match_type: "regex"
    on_failure: "abort"
  - name: "get_service_logs"
    ...
  # Continue with: capture logs, restart deployment, wait for rollout, verify health
```

Write at least 5 steps covering: check status, capture logs, restart, wait, verify.

<details><summary>Hint</summary>
Steps should be: (1) check current status, (2) capture logs for post-mortem, (3) restart the
deployment, (4) wait for rollout to complete, (5) verify the service is healthy. Each step
should have a verification check and an appropriate on_failure strategy.
</details>

### Part D: Add Error Handling and Edge Cases

Enhance the `AutomatedRunbook` class to handle these edge cases:

1. **Command timeout**: A command hangs for more than 30 seconds.
2. **Verification ambiguity**: The output does not match, but is not clearly wrong either.
3. **Rollback failure**: A rollback step itself fails.
4. **Partial execution**: Some steps succeeded, some failed -- produce a clear report.
5. **Concurrent steps**: (Bonus) Support steps that can run in parallel.

For each edge case, write a test that demonstrates the behavior.

```python
def test_command_timeout():
    """A hanging command should be killed after the timeout."""
    step = Step(
        name="hanging_step",
        description="This command will hang",
        command="sleep 300",
        verification=None,
        on_failure=OnFailure.ABORT,
        max_retries=0,
        timeout=2  # 2 second timeout for testing
    )
    runbook = Runbook(
        name="test",
        description="test",
        version="1.0",
        author="test",
        steps=[step]
    )
    executor = AutomatedRunbook(runbook)
    result = executor.execute()
    assert result is False
    log = executor.get_execution_log()
    assert log[0]["status"] == StepStatus.FAILED
    assert "timeout" in log[0]["error"].lower()
```

<details><summary>Hint</summary>
For timeout: use `subprocess.run(..., timeout=step.timeout)` and catch `subprocess.TimeoutExpired`.
For rollback failure: catch exceptions during rollback and log them, but continue rolling back
remaining steps. For partial execution: the execution log should clearly show which steps
succeeded and which failed.
</details>

## Success Criteria

- [ ] Data model (dataclasses) is complete and well-typed.
- [ ] `AutomatedRunbook.execute()` runs all steps in order with proper error handling.
- [ ] `dry_run` mode logs commands without executing them.
- [ ] Verification supports both exact match and regex patterns.
- [ ] Retry logic uses exponential backoff.
- [ ] Rollback executes in reverse order for all executed steps.
- [ ] Execution log captures all relevant details (command, output, status, duration, error).
- [ ] Command timeout is handled gracefully.
- [ ] Rollback failures are logged but do not crash the executor.
- [ ] Sample YAML runbook has at least 5 steps with realistic commands.
- [ ] At least 3 edge case tests are written and passing.

## What You Should Understand After This Exercise

Runbook automation transforms static documentation into executable code. The key design decisions
are: (1) how to verify success (exact match vs. regex vs. custom validator), (2) how to handle
failure (abort, retry, continue, rollback), and (3) how to produce a log that is useful for both
debugging the automation and post-incident review. The gap between "a human following steps" and
"code executing steps" is narrower than it appears -- the hard part is encoding the decision
logic that an experienced engineer applies intuitively.
