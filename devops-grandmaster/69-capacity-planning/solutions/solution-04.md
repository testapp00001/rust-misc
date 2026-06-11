# Solution 04: Growth Forecasting

## Part A: Linear Forecasting

```python
from dataclasses import dataclass
from datetime import datetime, timedelta
from typing import List, Tuple

@dataclass
class ForecastResult:
    current_value: float
    current_capacity: float
    utilization_today: float
    projected_exhaustion_date: datetime
    days_until_exhaustion: int
    recommended_action_date: datetime
    daily_growth_rate: float

def forecast_linear(
    data_points: List[Tuple[datetime, float]],
    current_capacity: float
) -> ForecastResult:
    """Linear growth forecast — assumes constant daily growth."""
    if len(data_points) < 2:
        raise ValueError("Need at least 2 data points")

    first_date, first_value = data_points[0]
    last_date, last_value = data_points[-1]
    days_elapsed = max((last_date - first_date).days, 1)

    # Linear growth rate: constant increase per day
    daily_growth = (last_value - first_value) / days_elapsed

    # Days until capacity is reached
    headroom = current_capacity - last_value
    if daily_growth <= 0:
        days_to_exhaustion = 9999
    else:
        days_to_exhaustion = headroom / daily_growth

    exhaustion_date = last_date + timedelta(days=days_to_exhaustion)
    action_date = exhaustion_date - timedelta(days=30)  # 30-day buffer

    return ForecastResult(
        current_value=last_value,
        current_capacity=current_capacity,
        utilization_today=(last_value / current_capacity) * 100,
        projected_exhaustion_date=exhaustion_date,
        days_until_exhaustion=int(days_to_exhaustion),
        recommended_action_date=max(action_date, datetime.now()),
        daily_growth_rate=daily_growth,
    )
```

**Why this works**: Linear forecasting assumes the growth rate is constant. It calculates `daily_growth = (last - first) / days` and projects forward: `days_to_exhaustion = (capacity - current) / daily_growth`. Simple but works well for steady, predictable growth.

---

## Part B: Exponential Forecasting

```python
import math

def forecast_exponential(
    data_points: List[Tuple[datetime, float]],
    current_capacity: float
) -> ForecastResult:
    """Exponential growth forecast — for accelerating growth."""
    if len(data_points) < 3:
        return forecast_linear(data_points, current_capacity)

    first_date, first_value = data_points[0]
    last_date, last_value = data_points[-1]
    days_elapsed = max((last_date - first_date).days, 1)

    # Exponential growth rate: rate = ln(last/first) / days
    if first_value <= 0:
        return forecast_linear(data_points, current_capacity)

    daily_growth_rate = math.log(last_value / first_value) / days_elapsed

    if daily_growth_rate <= 0:
        days_to_exhaustion = 9999
    else:
        # Days to reach capacity: ln(capacity/current) / rate
        days_to_exhaustion = math.log(current_capacity / last_value) / daily_growth_rate

    exhaustion_date = last_date + timedelta(days=days_to_exhaustion)
    action_date = exhaustion_date - timedelta(days=60)  # 60-day buffer for exponential

    return ForecastResult(
        current_value=last_value,
        current_capacity=current_capacity,
        utilization_today=(last_value / current_capacity) * 100,
        projected_exhaustion_date=exhaustion_date,
        days_until_exhaustion=int(days_to_exhaustion),
        recommended_action_date=max(action_date, datetime.now()),
        daily_growth_rate=daily_growth_rate,
    )
```

**Why this works**: Exponential forecasting models growth as `y = a * e^(r*t)`. The growth rate `r` is calculated from the ratio of first and last values: `r = ln(last/first) / days`. The exhaustion formula is `ln(capacity/current) / r`. This model is more accurate for rapidly growing systems because it accounts for the increasing growth rate.

---

## Part C: Compare the Models

Given the data:
```
Week 1:  2,000 RPS
Week 12: 4,400 RPS
Capacity: 8,000 RPS
```

### Linear Model
```
daily_growth = (4400 - 2000) / (11 * 7) = 2400 / 77 = 31.2 RPS/day
days_to_exhaustion = (8000 - 4400) / 31.2 = 115 days
exhaustion_date = Week 12 + 115 days ≈ Week 29
```

### Exponential Model
```
daily_growth_rate = ln(4400 / 2000) / 77 = 0.788 / 77 = 0.01023 per day
days_to_exhaustion = ln(8000 / 4400) / 0.01023 = 0.598 / 0.01023 = 58.4 days
exhaustion_date = Week 12 + 58 days ≈ Week 20
```

### Comparison

| Model | Exhaustion Date | Days Remaining | Action Date |
|-------|----------------|---------------|-------------|
| Linear | Week 29 | 115 days | Week 25 (30-day buffer) |
| Exponential | Week 20 | 58 days | Week 12 (60-day buffer) |

**The exponential model predicts earlier exhaustion (Week 20 vs Week 29).**

**Why the difference**: The data shows accelerating growth:
- Weeks 1-4: ~100 RPS/week increase
- Weeks 5-8: ~200 RPS/week increase
- Weeks 9-12: ~400 RPS/week increase

The linear model assumes constant growth (31 RPS/day). The exponential model captures the accelerating growth, which means capacity will be reached sooner.

**Which is more appropriate**: The exponential model is more appropriate because the growth is clearly accelerating. The linear model underestimates the urgency.

---

## Part D: Action Recommendations

```python
def generate_recommendations(forecast: ForecastResult, model_name: str) -> dict:
    """Generate actionable recommendations from forecast."""

    # Buffer: 30 days for linear, 60 days for exponential
    buffer_days = 30 if model_name == "linear" else 60
    action_date = forecast.projected_exhaustion_date - timedelta(days=buffer_days)

    # Target capacity: 20% headroom above projected need at action date
    days_until_action = max((action_date - datetime.now()).days, 0)
    projected_need_at_action = forecast.current_value + (
        forecast.daily_growth_rate * days_until_action
    )
    target_capacity = projected_need_at_action * 1.2

    # Cost estimate
    vcpus_needed = (target_capacity / 1000) * 4  # 4 vCPUs per 1000 RPS
    monthly_cost = vcpus_needed * 0.10 * 730      # $0.10/hr * 730 hrs/month

    return {
        "model": model_name,
        "action_by": action_date.strftime("%Y-%m-%d"),
        "target_capacity_rps": round(target_capacity),
        "additional_vcpus": round(vcpus_needed),
        "estimated_monthly_cost": round(monthly_cost, 2),
        "days_until_action": days_until_action,
    }
```

### Example Output

**Linear model:**
```
Action by:          2024-03-15 (30 days before exhaustion)
Target capacity:    6,240 RPS (20% headroom)
Additional vCPUs:   25
Monthly cost:       $182.50
```

**Exponential model:**
```
Action by:          2024-01-15 (60 days before exhaustion)
Target capacity:    7,800 RPS (20% headroom)
Additional vCPUs:   31
Monthly cost:       $226.30
```

**Why exponential needs more buffer**: Exponential growth accelerates. If you start capacity expansion too late, the growth outpaces your ability to provision. A 60-day buffer gives enough time to procure, configure, and test new infrastructure.

---

## Common Mistakes
1. **Using linear forecasting for accelerating growth**: Linear models underestimate growth when the rate is increasing. Always check if growth is accelerating before choosing a model.
2. **Insufficient buffer**: 30 days may not be enough for procurement, testing, and deployment. Use 60 days for exponential growth or when hardware procurement is involved.
3. **Not accounting for seasonality**: Weekly data may show weekend dips. Use peak values, not averages, for capacity planning.
4. **Projecting beyond the data**: Forecasting 6 months ahead from 3 months of data is unreliable. Re-forecast monthly with updated data.

## Relevant README Sections
- [Growth Forecasting](../README.md#step-4-growth-forecasting)
- [The Production Way](../README.md#the-production-way)
