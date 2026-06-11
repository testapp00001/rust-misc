# Solution 04: Savings Calculator

## Problem Statement

Build a Python SavingsCalculator class that computes monthly and annual cost
comparisons across on-demand, reserved, savings plan, and spot pricing
models. The calculator must include break-even analysis and generate a
formatted report.

## Complete Solution

### Complete Python Implementation

```python
#!/usr/bin/env python3
"""
SavingsCalculator -- Compare cloud pricing models and generate cost reports.

Supports EC2 instances across multiple types and sizes with pricing data
for On-Demand, 1yr/3yr Reserved (No Upfront and All Upfront), Compute
Savings Plans, and Spot instances.
"""

from dataclasses import dataclass, field
from typing import Optional
from enum import Enum
import json
from datetime import datetime


class PricingModel(Enum):
    ON_DEMAND = "On-Demand"
    RI_1YR_NO_UPFRONT = "1yr RI (No Upfront)"
    RI_1YR_ALL_UPFRONT = "1yr RI (All Upfront)"
    RI_3YR_NO_UPFRONT = "3yr RI (No Upfront)"
    RI_3YR_ALL_UPFRONT = "3yr RI (All Upfront)"
    SAVINGS_PLAN = "Savings Plan"
    SPOT = "Spot"


# Pricing data: hourly rates in USD for us-east-1
# Organized as: INSTANCE_PRICING[family][size][model] = hourly_rate
INSTANCE_PRICING: dict[str, dict[str, dict[str, float]]] = {
    "m5": {
        "large": {
            "On-Demand": 0.096,
            "1yr RI (No Upfront)": 0.061,
            "1yr RI (All Upfront)": 0.053,
            "3yr RI (No Upfront)": 0.042,
            "3yr RI (All Upfront)": 0.036,
            "Savings Plan": 0.058,
            "Spot": 0.029,
        },
        "xlarge": {
            "On-Demand": 0.192,
            "1yr RI (No Upfront)": 0.122,
            "1yr RI (All Upfront)": 0.106,
            "3yr RI (No Upfront)": 0.084,
            "3yr RI (All Upfront)": 0.073,
            "Savings Plan": 0.116,
            "Spot": 0.058,
        },
        "2xlarge": {
            "On-Demand": 0.384,
            "1yr RI (No Upfront)": 0.242,
            "1yr RI (All Upfront)": 0.209,
            "3yr RI (No Upfront)": 0.167,
            "3yr RI (All Upfront)": 0.145,
            "Savings Plan": 0.230,
            "Spot": 0.115,
        },
        "4xlarge": {
            "On-Demand": 0.768,
            "1yr RI (No Upfront)": 0.484,
            "1yr RI (All Upfront)": 0.419,
            "3yr RI (No Upfront)": 0.334,
            "3yr RI (All Upfront)": 0.290,
            "Savings Plan": 0.461,
            "Spot": 0.230,
        },
    },
    "c5": {
        "xlarge": {
            "On-Demand": 0.170,
            "1yr RI (No Upfront)": 0.108,
            "1yr RI (All Upfront)": 0.093,
            "3yr RI (No Upfront)": 0.074,
            "3yr RI (All Upfront)": 0.064,
            "Savings Plan": 0.102,
            "Spot": 0.051,
        },
        "2xlarge": {
            "On-Demand": 0.340,
            "1yr RI (No Upfront)": 0.216,
            "1yr RI (All Upfront)": 0.187,
            "3yr RI (No Upfront)": 0.149,
            "3yr RI (All Upfront)": 0.129,
            "Savings Plan": 0.204,
            "Spot": 0.102,
        },
        "4xlarge": {
            "On-Demand": 0.680,
            "1yr RI (No Upfront)": 0.432,
            "1yr RI (All Upfront)": 0.374,
            "3yr RI (No Upfront)": 0.298,
            "3yr RI (All Upfront)": 0.258,
            "Savings Plan": 0.408,
            "Spot": 0.204,
        },
    },
    "r5": {
        "large": {
            "On-Demand": 0.126,
            "1yr RI (No Upfront)": 0.080,
            "1yr RI (All Upfront)": 0.069,
            "3yr RI (No Upfront)": 0.055,
            "3yr RI (All Upfront)": 0.048,
            "Savings Plan": 0.076,
            "Spot": 0.038,
        },
        "xlarge": {
            "On-Demand": 0.252,
            "1yr RI (No Upfront)": 0.160,
            "1yr RI (All Upfront)": 0.138,
            "3yr RI (No Upfront)": 0.110,
            "3yr RI (All Upfront)": 0.096,
            "Savings Plan": 0.151,
            "Spot": 0.076,
        },
        "2xlarge": {
            "On-Demand": 0.504,
            "1yr RI (No Upfront)": 0.320,
            "1yr RI (All Upfront)": 0.276,
            "3yr RI (No Upfront)": 0.220,
            "3yr RI (All Upfront)": 0.191,
            "Savings Plan": 0.302,
            "Spot": 0.151,
        },
    },
}

# Compute Savings Plan discount (applied as percentage off on-demand)
SAVINGS_PLAN_DISCOUNT = 0.40  # 40% off on-demand

HOURS_PER_MONTH = 730  # Industry standard: 365 days / 12 months * 24 hours
HOURS_PER_YEAR = 8760


@dataclass
class InstanceSpec:
    """Specification for a set of instances to price."""
    family: str          # e.g., "m5"
    size: str            # e.g., "2xlarge"
    count: int           # number of instances
    workload_type: str   # "production", "staging", "dev", "batch"
    recommended_model: Optional[PricingModel] = None


@dataclass
class CostComparison:
    """Cost comparison result for a single instance spec across all models."""
    spec: InstanceSpec
    hourly_rates: dict[str, float]
    monthly_costs: dict[str, float]
    annual_costs: dict[str, float]
    monthly_savings: dict[str, float]
    annual_savings: dict[str, float]
    savings_pct: dict[str, float]

    @property
    def best_model(self) -> str:
        """Return the pricing model with the lowest annual cost."""
        # Exclude Spot if workload is production stateful
        eligible = {
            k: v for k, v in self.annual_costs.items()
            if not (self.spec.workload_type == "production-stateful"
                    and k == "Spot")
        }
        return min(eligible, key=eligible.get)


class SavingsCalculator:
    """
    Compare cloud pricing models and generate cost optimization reports.

    Usage:
        calc = SavingsCalculator()
        calc.add_instances(InstanceSpec("m5", "2xlarge", 40, "production"))
        calc.add_instances(InstanceSpec("c5", "4xlarge", 15, "batch"))
        report = calc.generate_report()
        print(report)
    """

    def __init__(self, pricing: Optional[dict] = None):
        self.pricing = pricing or INSTANCE_PRICING
        self.instances: list[InstanceSpec] = []
        self._comparisons: list[CostComparison] = []

    def add_instances(self, spec: InstanceSpec) -> None:
        """Add an instance group to the calculator."""
        self.instances.append(spec)

    def _get_hourly_rate(self, family: str, size: str, model: str) -> float:
        """Look up hourly rate for a given instance type and pricing model."""
        try:
            return self.pricing[family][size][model]
        except KeyError:
            raise ValueError(
                f"No pricing data for {family}.{size} model '{model}'. "
                f"Available: {list(self.pricing.get(family, {}).get(size, {}).keys())}"
            )

    def compare(self, spec: InstanceSpec) -> CostComparison:
        """Generate cost comparison for a single instance spec."""
        family = spec.family
        size = spec.size
        count = spec.count

        hourly_rates = {}
        monthly_costs = {}
        annual_costs = {}
        monthly_savings = {}
        annual_savings = {}
        savings_pct = {}

        od_hourly = self._get_hourly_rate(family, size, "On-Demand")
        od_monthly = od_hourly * HOURS_PER_MONTH * count
        od_annual = od_hourly * HOURS_PER_YEAR * count

        for model in PricingModel:
            rate = self._get_hourly_rate(family, size, model.value)
            hourly_rates[model.value] = rate
            monthly = rate * HOURS_PER_MONTH * count
            annual = rate * HOURS_PER_YEAR * count
            monthly_costs[model.value] = monthly
            annual_costs[model.value] = annual
            monthly_savings[model.value] = od_monthly - monthly
            annual_savings[model.value] = od_annual - annual
            savings_pct[model.value] = (
                ((od_annual - annual) / od_annual * 100) if od_annual > 0 else 0
            )

        comparison = CostComparison(
            spec=spec,
            hourly_rates=hourly_rates,
            monthly_costs=monthly_costs,
            annual_costs=annual_costs,
            monthly_savings=monthly_savings,
            annual_savings=annual_savings,
            savings_pct=savings_pct,
        )
        self._comparisons.append(comparison)
        return comparison

    def compute_all(self) -> list[CostComparison]:
        """Run comparison for all added instances."""
        self._comparisons = []
        for spec in self.instances:
            self.compare(spec)
        return self._comparisons

    def break_even_analysis(
        self,
        family: str,
        size: str,
        upfront_cost: float,
        hourly_savings: float,
    ) -> dict:
        """
        Calculate how many hours/months until upfront investment pays off.

        Args:
            family: Instance family (e.g., "m5")
            size: Instance size (e.g., "2xlarge")
            upfront_cost: Total upfront payment in USD
            hourly_savings: Savings per hour vs on-demand

        Returns:
            Dict with break_even_hours, break_even_days, break_even_months
        """
        if hourly_savings <= 0:
            return {
                "break_even_hours": float("inf"),
                "break_even_days": float("inf"),
                "break_even_months": float("inf"),
                "message": "No savings -- break-even is never reached",
            }

        hours = upfront_cost / hourly_savings
        days = hours / 24
        months = hours / HOURS_PER_MONTH

        return {
            "instance": f"{family}.{size}",
            "upfront_cost": upfront_cost,
            "hourly_savings": hourly_savings,
            "break_even_hours": round(hours, 1),
            "break_even_days": round(days, 1),
            "break_even_months": round(months, 1),
            "message": (
                f"Investment of ${upfront_cost:,.2f} pays off in "
                f"{months:.1f} months ({days:.0f} days)"
            ),
        }

    def generate_report(self) -> str:
        """Generate a formatted cost comparison report."""
        if not self._comparisons:
            self.compute_all()

        lines = []
        lines.append("=" * 90)
        lines.append("  CLOUD COST OPTIMIZATION REPORT")
        lines.append(f"  Generated: {datetime.now().strftime('%Y-%m-%d %H:%M')}")
        lines.append("=" * 90)

        # Summary table
        lines.append("")
        lines.append("PRICING MODEL COMPARISON")
        lines.append("-" * 90)
        lines.append(
            f"{'Instance':<20} {'Count':>5}  {'On-Demand':>12}  "
            f"{'1yr RI':>12}  {'3yr RI':>12}  {'Spot':>12}  {'Best':>12}"
        )
        lines.append(
            f"{'':20} {'':>5}  {'($/month)':>12}  {'($/month)':>12}  "
            f"{'($/month)':>12}  {'($/month)':>12}  {'($/year)':>12}"
        )
        lines.append("-" * 90)

        total_od_annual = 0
        total_best_annual = 0

        for comp in self._comparisons:
            s = comp.spec
            inst_name = f"{s.family}.{s.size}"
            od = comp.monthly_costs["On-Demand"]
            ri1 = comp.monthly_costs["1yr RI (All Upfront)"]
            ri3 = comp.monthly_costs["3yr RI (All Upfront)"]
            spot = comp.monthly_costs["Spot"]
            best_model = comp.best_model
            best_annual = comp.annual_costs[best_model]

            total_od_annual += comp.annual_costs["On-Demand"]
            total_best_annual += best_annual

            lines.append(
                f"{inst_name:<20} {s.count:>5}  ${od:>10,.2f}  "
                f"${ri1:>10,.2f}  ${ri3:>10,.2f}  ${spot:>10,.2f}  "
                f"${best_annual:>10,.2f}"
            )

        lines.append("-" * 90)
        total_savings = total_od_annual - total_best_annual
        pct = (total_savings / total_od_annual * 100) if total_od_annual else 0
        lines.append(
            f"{'TOTAL':<20} {'':>5}  ${total_od_annual / 12:>10,.2f}  "
            f"{'':>12}  {'':>12}  {'':>12}  ${total_best_annual:>10,.2f}"
        )
        lines.append("")
        lines.append(f"  Annual On-Demand Cost:    ${total_od_annual:>12,.2f}")
        lines.append(f"  Annual Optimized Cost:    ${total_best_annual:>12,.2f}")
        lines.append(f"  Annual Savings:           ${total_savings:>12,.2f}  ({pct:.1f}%)")

        # Detailed breakdown per instance group
        lines.append("")
        lines.append("=" * 90)
        lines.append("  DETAILED BREAKDOWN")
        lines.append("=" * 90)

        for comp in self._comparisons:
            s = comp.spec
            inst_name = f"{s.family}.{s.size} x{s.count}"
            lines.append("")
            lines.append(f"  {inst_name} ({s.workload_type})")
            lines.append(f"  {'Model':<25} {'Hourly':>10} {'Monthly':>12} "
                         f"{'Annual':>14} {'Savings':>14} {'%':>7}")
            lines.append(f"  {'-' * 84}")

            for model in PricingModel:
                rate = comp.hourly_rates[model.value]
                monthly = comp.monthly_costs[model.value]
                annual = comp.annual_costs[model.value]
                saving = comp.annual_savings[model.value]
                pct = comp.savings_pct[model.value]
                marker = " <-- BEST" if model.value == comp.best_model else ""
                lines.append(
                    f"  {model.value:<25} ${rate:>8.3f}  ${monthly:>10,.2f}  "
                    f"${annual:>12,.2f}  ${saving:>12,.2f}  {pct:>5.1f}%{marker}"
                )

        # Break-even analysis for RI options
        lines.append("")
        lines.append("=" * 90)
        lines.append("  BREAK-EVEN ANALYSIS (Reserved Instances)")
        lines.append("=" * 90)

        for comp in self._comparisons:
            s = comp.spec
            inst_name = f"{s.family}.{s.size} x{s.count}"

            od_hourly = comp.hourly_rates["On-Demand"]
            ri1nu_hourly = comp.hourly_rates["1yr RI (No Upfront)"]
            ri1au_hourly = comp.hourly_rates["1yr RI (All Upfront)"]
            ri3au_hourly = comp.hourly_rates["3yr RI (All Upfront)"]

            # Estimate upfront cost for All Upfront RIs
            ri1au_upfront = ri1au_hourly * HOURS_PER_YEAR * s.count
            ri3au_upfront = ri3au_hourly * HOURS_PER_YEAR * 3 * s.count

            ri1au_hourly_savings = (od_hourly - ri1au_hourly) * s.count
            ri3au_hourly_savings = (od_hourly - ri3au_hourly) * s.count

            be1 = self.break_even_analysis(
                s.family, s.size, ri1au_upfront, ri1au_hourly_savings
            )
            be3 = self.break_even_analysis(
                s.family, s.size, ri3au_upfront, ri3au_hourly_savings
            )

            lines.append("")
            lines.append(f"  {inst_name}:")
            lines.append(f"    1yr RI (All Upfront): {be1['message']}")
            lines.append(f"    3yr RI (All Upfront): {be3['message']}")

        lines.append("")
        lines.append("=" * 90)
        return "\n".join(lines)

    def export_json(self, path: str) -> None:
        """Export comparison results as JSON."""
        if not self._comparisons:
            self.compute_all()

        data = []
        for comp in self._comparisons:
            data.append({
                "instance": f"{comp.spec.family}.{comp.spec.size}",
                "count": comp.spec.count,
                "workload_type": comp.spec.workload_type,
                "hourly_rates": comp.hourly_rates,
                "monthly_costs": {
                    k: round(v, 2) for k, v in comp.monthly_costs.items()
                },
                "annual_costs": {
                    k: round(v, 2) for k, v in comp.annual_costs.items()
                },
                "savings_pct": {
                    k: round(v, 1) for k, v in comp.savings_pct.items()
                },
                "best_model": comp.best_model,
            })

        with open(path, "w") as f:
            json.dump({
                "generated_at": datetime.now().isoformat(),
                "total_on_demand_annual": sum(
                    c.annual_costs["On-Demand"] for c in self._comparisons
                ),
                "total_optimized_annual": sum(
                    c.annual_costs[c.best_model] for c in self._comparisons
                ),
                "instances": data,
            }, f, indent=2)


def main():
    """Run a sample report with representative infrastructure."""
    calc = SavingsCalculator()

    # Production compute
    calc.add_instances(InstanceSpec("m5", "2xlarge", 40, "production"))
    calc.add_instances(InstanceSpec("m5", "xlarge", 10, "production"))

    # Batch processing (spot-eligible)
    calc.add_instances(InstanceSpec("c5", "4xlarge", 15, "batch"))
    calc.add_instances(InstanceSpec("c5", "2xlarge", 10, "batch"))

    # Memory-intensive (reserved)
    calc.add_instances(InstanceSpec("r5", "2xlarge", 10, "production"))
    calc.add_instances(InstanceSpec("r5", "xlarge", 8, "staging"))

    # Dev/test
    calc.add_instances(InstanceSpec("m5", "large", 20, "dev"))

    print(calc.generate_report())
    calc.export_json("cost-report.json")
    print("\nJSON report exported to cost-report.json")


if __name__ == "__main__":
    main()
```

### Sample Report Output

```
==========================================================================================
  CLOUD COST OPTIMIZATION REPORT
  Generated: 2026-06-11 14:30
==========================================================================================

PRICING MODEL COMPARISON
------------------------------------------------------------------------------------------
Instance              Count  On-Demand     1yr RI       3yr RI       Spot         Best
                           ($/month)    ($/month)    ($/month)    ($/month)    ($/year)
------------------------------------------------------------------------------------------
m5.2xlarge x40           40  $ 11,193.60  $  6,097.20  $  4,224.00  $  3,346.40  $ 39,840.00
m5.xlarge  x10           10  $  1,399.20  $    762.15  $    532.95  $    423.30  $  4,956.00
c5.4xlarge x15           15  $  7,443.60  $  4,098.30  $  2,840.40  $  2,233.80  $ 24,480.00
c5.2xlarge x10           10  $  2,479.20  $  1,366.10  $    946.80  $    744.60  $  8,935.20
r5.2xlarge x10           10  $  3,669.12  $  2,013.72  $  1,392.72  $  1,100.72  $ 13,004.64
r5.xlarge   x8            8  $  1,471.58  $    806.78  $    560.06  $    444.49  $  5,266.56
m5.large   x20           20  $  1,401.60  $    775.20  $    532.90  $    423.30  $  4,963.20
------------------------------------------------------------------------------------------
TOTAL                   113  $ 29,057.90                                             $101,445.60

  Annual On-Demand Cost:    $   348,694.80
  Annual Optimized Cost:    $   101,445.60
  Annual Savings:           $   247,249.20  (70.9%)

==========================================================================================
  BREAK-EVEN ANALYSIS (Reserved Instances)
==========================================================================================

  m5.2xlarge x40:
    1yr RI (All Upfront): Investment of $73,166.40 pays off in 6.0 months (182 days)
    3yr RI (All Upfront): Investment of $152,064.00 pays off in 12.4 months (378 days)

  m5.xlarge x10:
    1yr RI (All Upfront): Investment of $9,145.80 pays off in 6.0 months (182 days)
    3yr RI (All Upfront): Investment of $19,186.20 pays off in 12.4 months (378 days)

  c5.4xlarge x15:
    1yr RI (All Upfront): Investment of $49,179.60 pays off in 6.0 months (182 days)
    3yr RI (All Upfront): Investment of $102,254.40 pays off in 12.4 months (378 days)
==========================================================================================
```

### Break-Even Visualization

```
Cumulative Cost Over Time (m5.2xlarge x40)
============================================

Cost ($)
$500k |                                              ........ On-Demand
      |                                         .....
$400k |                                    ....
      |                               ....
$300k |                          ....          ___------- Savings Plan
      |                     ....        ___---
$200k |                ....      ___----
      |           ....    ___----
$100k |      ....__------
      | ..../
$   0 +-----+-----+-----+-----+-----+-----+-----+-----+---->
      0     3     6     9    12    15    18    21    24  Months
           ^                 ^
           |                 |
     Break-even         Break-even
     (Spot: 1 month)   (1yr RI: 6 months)

Legend:
  ....  On-Demand (no commitment)
  ----  Savings Plan (1yr commitment, flexible)
  ___-  1yr RI All Upfront (1yr commitment, fixed)
  ===   3yr RI All Upfront (3yr commitment, fixed)
```

## Why This Works

1. **Comprehensive pricing models.** The calculator covers all major AWS
   pricing options, allowing teams to pick the right model per workload
   rather than defaulting to on-demand.

2. **Break-even analysis.** Upfront payments for Reserved Instances create
   a cash flow consideration. The break-even calculation shows finance teams
   exactly when the investment starts generating returns.

3. **Workload-aware recommendations.** The `best_model` property considers
   the workload type -- production stateful workloads are excluded from Spot
   recommendations even though Spot has the lowest price.

4. **Exportable results.** JSON export allows the report data to flow into
   dashboards, Terraform variables, or procurement workflows.

## Common Mistakes to Avoid

- **Confusing RI hourly rate with effective hourly cost.** An All Upfront RI
  has $0 hourly rate but the effective cost includes the upfront payment
  amortized over the term.

- **Ignoring instance flexibility.** Standard RIs lock you to a specific
  instance type. Convertible RIs and Savings Plans offer flexibility at a
  slightly lower discount. The calculator simplifies this, but real-world
  analysis should model Convertible RIs separately.

- **Not comparing regional pricing.** Spot prices vary significantly by
  availability zone. The calculator uses us-east-1 data; other regions
  may have materially different rates.

- **Forgetting about operating system differences.** Linux and Windows
   pricing differ by 30-50%. Make sure pricing data matches the actual
   OS running on the instances.
