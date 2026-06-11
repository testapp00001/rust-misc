# Exercise 04: Savings Calculator

**Type:** Challenge | **Time:** 45 min | **Difficulty:** Medium-Hard

## Objective

Build a Python `SavingsCalculator` class that compares on-demand, reserved (1-year and
3-year), and spot pricing for multiple instance types, calculates break-even points, and
generates optimization reports.

## Background

Your organization wants a reusable tool to evaluate cloud compute costs across different
pricing models. The tool should help engineering teams make data-driven decisions about
which pricing model to use for their workloads.

## Requirements

Build a `SavingsCalculator` class in Python that supports:

1. Loading instance pricing data (on-demand, reserved 1yr, reserved 3yr, spot)
2. Calculating total cost over a given time period
3. Finding break-even points between on-demand and reserved instances
4. Generating optimization recommendations
5. Outputting a formatted report

## Pricing Data

Use these representative AWS prices (us-east-1, Linux, per-hour):

| Instance Type | On-Demand | RI 1yr (No Upfront) | RI 3yr (All Upfront) | Spot (avg) | vCPUs | RAM (GiB) |
|--------------|-----------|---------------------|----------------------|------------|-------|-----------|
| m5.large | $0.096 | $0.061 | $0.038 | $0.029 | 2 | 8 |
| m5.xlarge | $0.192 | $0.122 | $0.076 | $0.058 | 4 | 16 |
| m5.2xlarge | $0.384 | $0.244 | $0.153 | $0.115 | 8 | 32 |
| m5.4xlarge | $0.768 | $0.489 | $0.305 | $0.230 | 16 | 64 |
| r5.xlarge | $0.252 | $0.161 | $0.101 | $0.076 | 4 | 32 |
| r5.2xlarge | $0.504 | $0.322 | $0.201 | $0.151 | 8 | 64 |
| c5.xlarge | $0.170 | $0.109 | $0.068 | $0.051 | 4 | 8 |
| c5.2xlarge | $0.340 | $0.218 | $0.136 | $0.102 | 8 | 16 |
| p3.2xlarge | $3.060 | $1.958 | $1.224 | $0.918 | 8 | 61 |

## Tasks

### Task 1: Implement the SavingsCalculator Class

```python
class SavingsCalculator:
    """Compare cloud compute pricing models and generate optimization reports."""

    def __init__(self):
        """Initialize with pricing data."""
        # Store pricing data as a dictionary
        # Keys: instance type
        # Values: dict with 'on_demand', 'ri_1yr', 'ri_3yr', 'spot', 'vcpus', 'ram'
        self.pricing = {}
        self._load_default_pricing()

    def _load_default_pricing(self):
        """Load the default AWS pricing table."""
        # IMPLEMENT THIS
        pass

    def add_instance(self, instance_type, on_demand, ri_1yr, ri_3yr, spot, vcpus, ram):
        """Add or update pricing for an instance type."""
        # IMPLEMENT THIS
        pass

    def calculate_monthly_cost(self, instance_type, count, pricing_model, hours_per_month=730):
        """Calculate the monthly cost for a given instance type and count.

        Args:
            instance_type: The EC2 instance type (e.g., 'm5.xlarge')
            count: Number of instances
            pricing_model: One of 'on_demand', 'ri_1yr', 'ri_3yr', 'spot'
            hours_per_month: Hours in a billing month (default 730)

        Returns:
            Monthly cost in dollars

        Raises:
            ValueError: If instance_type or pricing_model is invalid
        """
        # IMPLEMENT THIS
        pass

    def calculate_total_cost(self, workloads, months=12):
        """Calculate total cost for a list of workloads over a time period.

        Args:
            workloads: List of dicts, each with:
                - 'instance_type': str
                - 'count': int
                - 'pricing_model': str
                - 'hours_per_month': int (optional, default 730)
            months: Number of months to calculate for

        Returns:
            Total cost in dollars
        """
        # IMPLEMENT THIS
        pass

    def break_even_months(self, instance_type, count, pricing_model,
                          hours_per_month=730):
        """Calculate how many months until a pricing model breaks even vs on-demand.

        For RI: how many months until RI total cost < on-demand total cost,
        accounting for upfront payments.
        For spot: always cheaper per-hour, so break-even is 0.

        Args:
            instance_type: The EC2 instance type
            count: Number of instances
            pricing_model: 'ri_1yr', 'ri_3yr', or 'spot'
            hours_per_month: Hours in a billing month

        Returns:
            Number of months (float) to break even, or 0 if already cheaper
        """
        # IMPLEMENT THIS
        # Hint: RI 1yr has no upfront, RI 3yr has all upfront
        # For RI 3yr: break_even is when total RI cost (including upfront) < total OD cost
        # The RI 3yr price IS the amortized hourly rate (all upfront already paid)
        pass

    def recommend(self, workloads, analysis_months=12):
        """Generate pricing model recommendations for a list of workloads.

        For each workload, recommend the cheapest model given the time horizon.
        Consider:
            - Spot: cheapest but risky (flag if usage > 80% of hours)
            - RI 3yr: cheapest reserved, but long commitment
            - RI 1yr: moderate savings, shorter commitment
            - On-Demand: most flexible, most expensive

        Args:
            workloads: List of dicts with 'instance_type', 'count', 'hours_per_month'
            analysis_months: Time horizon in months

        Returns:
            List of dicts with recommendations:
            [
                {
                    'instance_type': str,
                    'count': int,
                    'current_model': 'on_demand',
                    'recommended_model': str,
                    'monthly_savings': float,
                    'annual_savings': float,
                    'savings_percent': float,
                    'notes': str
                },
                ...
            ]
        """
        # IMPLEMENT THIS
        pass

    def generate_report(self, workloads, analysis_months=12):
        """Generate a formatted text report comparing pricing models.

        The report should include:
        1. Header with date and analysis period
        2. Per-workload cost comparison table
        3. Summary with total costs per model
        4. Recommendations with savings
        5. Break-even analysis for recommended changes

        Args:
            workloads: List of workload dicts
            analysis_months: Time horizon in months

        Returns:
            Formatted string report
        """
        # IMPLEMENT THIS
        pass
```

### Task 2: Write Unit Tests

Write tests for the `SavingsCalculator` class covering:

```python
import unittest

class TestSavingsCalculator(unittest.TestCase):

    def setUp(self):
        """Create a fresh calculator for each test."""
        self.calc = SavingsCalculator()

    def test_monthly_cost_on_demand(self):
        """Test on-demand cost calculation for m5.xlarge x 4."""
        # 4 x m5.xlarge on-demand = 4 * 0.192 * 730 = $560.64
        cost = self.calc.calculate_monthly_cost('m5.xlarge', 4, 'on_demand')
        self.assertAlmostEqual(cost, 560.64, places=2)

    def test_monthly_cost_spot(self):
        """Test spot cost calculation for m5.xlarge x 4."""
        cost = self.calc.calculate_monthly_cost('m5.xlarge', 4, 'spot')
        # IMPLEMENT: calculate expected value and assert
        pass

    def test_monthly_cost_invalid_type(self):
        """Test that invalid instance type raises ValueError."""
        # IMPLEMENT THIS
        pass

    def test_monthly_cost_invalid_model(self):
        """Test that invalid pricing model raises ValueError."""
        # IMPLEMENT THIS
        pass

    def test_total_cost_mixed_workloads(self):
        """Test total cost calculation with multiple workloads."""
        workloads = [
            {'instance_type': 'm5.xlarge', 'count': 4, 'pricing_model': 'on_demand'},
            {'instance_type': 'c5.2xlarge', 'count': 2, 'pricing_model': 'spot'},
        ]
        total = self.calc.calculate_total_cost(workloads, months=1)
        # IMPLEMENT: calculate expected value and assert
        pass

    def test_break_even_spot(self):
        """Spot should break even immediately (month 0)."""
        months = self.calc.break_even_months('m5.xlarge', 4, 'spot')
        self.assertEqual(months, 0)

    def test_break_even_ri_1yr(self):
        """RI 1yr break-even should be immediate (no upfront)."""
        months = self.calc.break_even_months('m5.xlarge', 4, 'ri_1yr')
        # IMPLEMENT: RI 1yr has no upfront, so per-hour rate is lower from day 1
        pass

    def test_recommend_spot_for_batch(self):
        """Batch workloads should be recommended spot."""
        workloads = [
            {'instance_type': 'c5.2xlarge', 'count': 4, 'hours_per_month': 180},
        ]
        recs = self.calc.recommend(workloads)
        # IMPLEMENT: batch workload (180/730 hours) should recommend spot
        pass

    def test_report_contains_sections(self):
        """Report should contain expected sections."""
        workloads = [
            {'instance_type': 'm5.xlarge', 'count': 4, 'pricing_model': 'on_demand'},
        ]
        report = self.calc.generate_report(workloads)
        self.assertIn('Cost Analysis Report', report)
        self.assertIn('Recommendations', report)

if __name__ == '__main__':
    unittest.main()
```

### Task 3: Run the Calculator

Create a `main` function that demonstrates the calculator with this workload:

```python
def main():
    calc = SavingsCalculator()

    workloads = [
        {'instance_type': 'm5.2xlarge', 'count': 8, 'pricing_model': 'on_demand',
         'hours_per_month': 730},
        {'instance_type': 'm5.xlarge', 'count': 4, 'pricing_model': 'on_demand',
         'hours_per_month': 730},
        {'instance_type': 'r5.2xlarge', 'count': 6, 'pricing_model': 'on_demand',
         'hours_per_month': 730},
        {'instance_type': 'p3.2xlarge', 'count': 12, 'pricing_model': 'on_demand',
         'hours_per_month': 400},  # ML training, ~13 hrs/day
        {'instance_type': 'c5.2xlarge', 'count': 10, 'pricing_model': 'on_demand',
         'hours_per_month': 200},  # CI/CD, ~7 hrs/day
    ]

    report = calc.generate_report(workloads, analysis_months=12)
    print(report)

if __name__ == '__main__':
    main()
```

Run the calculator and capture the output. The report should show the total current cost
and the optimized cost with recommendations.

### Task 4: Enhance the Calculator

Add at least one of these enhancements:

**Option A: Capacity Reservations**
Add support for a `ri_1yr_partial` model (RI 1yr, partial upfront) with a different
discount rate. Update break-even calculations accordingly.

**Option B: Savings Plans**
Add an `ec2_savings_plan` model that applies a percentage discount across all instance
types in a family. Compare it against instance-specific RIs.

**Option C: Multi-Region Pricing**
Allow the calculator to accept region-specific pricing and compare costs across regions.
Some regions are 10-20% cheaper than us-east-1.

<details>
<summary>Hint 1: Break-Even for RI 3yr (All Upfront)</summary>

For "All Upfront" RI, you pay the entire 3-year cost upfront. The hourly rate in the
pricing table is the amortized rate. Break-even calculation:

```
ri_3yr_total = ri_hourly_rate * hours_per_month * 36
on_demand_total = od_hourly_rate * hours_per_month * 36
```

Since the RI rate is lower, break-even is essentially immediate (the commitment is the
risk, not the cost). The real question is: "Is the upfront payment worth the savings?"

For partial or no upfront RIs, break-even is when cumulative RI payments < cumulative
on-demand payments.

</details>

<details>
<summary>Hint 2: Report Format</summary>

```
================================================================================
                      COST OPTIMIZATION REPORT
                      Generated: 2026-06-11
                      Analysis Period: 12 months
================================================================================

WORKLOAD COST COMPARISON
--------------------------------------------------------------------------------
Instance Type  | Count | On-Demand   | RI 1yr      | RI 3yr      | Spot
m5.2xlarge     |     8 | $27,013.12  | $17,225.28  | $10,806.72  | $8,104.32
...

TOTALS
--------------------------------------------------------------------------------
On-Demand:  $???
RI 1yr:     $??? (save ???%)
RI 3yr:     $??? (save ???%)
Spot:       $??? (save ???%)

RECOMMENDATIONS
--------------------------------------------------------------------------------
1. m5.2xlarge x8: Switch to RI 1yr -> Save $X/month ($Y/year)
2. p3.2xlarge x12: Switch to Spot -> Save $X/month ($Y/year)
...
================================================================================
```

</details>

<details>
<summary>Hint 3: Testing Break-Even</summary>

For RI 1yr (no upfront), break-even is month 0 because the hourly rate is immediately
lower. But if there were upfront costs:
```
break_even = upfront_payment / (od_monthly - ri_monthly)
```
Where `od_monthly` and `ri_monthly` include the hourly rate difference times hours per
month.

</details>

## Verification

After completing this exercise, you should have:
- A working `SavingsCalculator` class with all required methods
- At least 8 passing unit tests
- A generated report showing current vs optimized costs
- At least one enhancement (Options A, B, or C)
- The main function output demonstrating savings of 30-50%

## Reflection Questions

1. Why might actual spot prices differ from the average, and how would you handle that?
2. What factors beyond hourly price should influence the on-demand vs reserved decision?
3. How would you integrate this calculator into a CI/CD pipeline to flag cost regressions?
