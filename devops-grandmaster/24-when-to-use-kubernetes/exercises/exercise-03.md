# Exercise 03: Build Your Organization's Decision Framework

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

Create a reusable decision framework that your organization can use to evaluate whether any new application or service should be deployed on Kubernetes, an alternative platform, or a simpler approach. This framework should be objective, repeatable, and based on measurable criteria.

## Scenario / Starting Point

Your engineering organization has 12 different applications running on a mix of Heroku, bare EC2 instances, Docker Compose on VPS servers, and one legacy application on a shared hosting provider. Teams keep making ad-hoc decisions about where to deploy, leading to inconsistent infrastructure, difficulty sharing knowledge, and unpredictable costs.

The VP of Engineering has asked you to create a decision framework that any team can use to evaluate their deployment options. The framework must be:

- **Objective** -- based on measurable criteria, not opinions
- **Repeatable** -- different people evaluating the same app should reach the same conclusion
- **Practical** -- a team can complete the evaluation in under 30 minutes

## Tasks

### Part A: Define the Criteria

Create a set of measurable criteria that determine the right deployment approach. For each criterion, define:

1. The question to ask
2. How to measure it (numeric thresholds, yes/no, or scale)
3. Which deployment approach each answer points toward

You need at least 8 criteria. The framework should cover:

- Application complexity
- Team structure
- Traffic patterns
- Compliance requirements
- Budget constraints
- Operational maturity

<details>
<summary>Hint 1</summary>

Good criteria are specific and measurable. Bad: "Is the application complex?" Good: "How many independent services does the application have?" Use numeric thresholds wherever possible (e.g., "fewer than 5 services" vs "more than 15 services").

</details>

### Part B: Build the Scoring System

Create a scoring system that takes the criteria from Part A and produces a recommendation. Your system should:

1. Assign points or weights to each criterion
2. Define score ranges that map to deployment approaches
3. Include any "hard stops" (criteria that override the score)

Fill in this template:

```
Scoring System:
- Criterion 1: [question] -> [points for each answer]
- Criterion 2: [question] -> [points for each answer]
- ...

Score Ranges:
- 0-X points: [approach]
- X-Y points: [approach]
- Y-Z points: [approach]

Hard Stops (override the score):
- [condition] -> forces [approach]
- [condition] -> forces [approach]
```

<details>
<summary>Hint 2</summary>

Hard stops are criteria that override the scoring system regardless of other factors. For example: "PCI DSS compliance required" might force Kubernetes or a managed platform regardless of team size. "Budget under $100/month" might force Docker Compose regardless of complexity.

</details>

### Part C: Validate with Test Cases

Apply your framework to these 5 test cases. For each one, show your scoring work and final recommendation.

**Test Case 1: Startup MVP**
- 2 developers
- 1 API + 1 database
- 100 users
- No compliance requirements
- Budget: $50/month
- First deployment ever

**Test Case 2: Growing SaaS**
- 8 developers in 2 teams
- 5 microservices + 2 databases
- 100,000 users, traffic doubles every quarter
- SOC 2 required in 6 months
- Budget: $2,000/month
- Deploy 5 times/week

**Test Case 3: Internal Tool**
- 1 developer (part-time)
- 1 web app + 1 database
- 30 internal users
- No compliance
- Budget: $100/month
- Deploy monthly

**Test Case 4: High-Traffic Platform**
- 25 developers in 6 teams
- 40 microservices
- 5 million users, 10x traffic spikes
- GDPR, SOC 2, HIPAA
- Budget: $15,000/month
- Deploy 30 times/day

**Test Case 5: Batch Processing**
- 3 developers
- 3 services (scheduler, worker, storage)
- Runs 8 hours/day, idle 16 hours
- No compliance
- Budget: $500/month
- Deploy weekly

<details>
<summary>Hint 3</summary>

A good framework should make the answer obvious for clear cases (Test 1, Test 4) and provide a structured way to decide for borderline cases (Test 2, Test 5). If your framework gives ambiguous results for clear cases, revise your criteria or thresholds.

</details>

### Part D: Edge Cases and Exceptions

Identify 3 situations where your framework might give a wrong recommendation. For each one, explain:

1. What the framework recommends
2. Why that recommendation might be wrong
3. What additional context a human decision-maker should consider

<details>
<summary>Hint 4</summary>

No framework is perfect. Common edge cases include: applications about to scale dramatically, teams with unique expertise (e.g., a team of Kubernetes experts joining a small company), and applications with unusual technical requirements (e.g., GPU workloads, real-time systems).

</details>

## Success Criteria

- [ ] Part A defines at least 8 measurable criteria covering all required dimensions.
- [ ] Each criterion has a specific question, measurement method, and mapping to deployment approaches.
- [ ] The scoring system in Part B produces a clear recommendation for each score range.
- [ ] At least 2 hard stops are defined that override the scoring system.
- [ ] Part C test cases are evaluated with shown scoring work and the recommendations are reasonable.
- [ ] Part D identifies at least 3 genuine edge cases with thoughtful analysis.
- [ ] The entire framework can be completed by a team in under 30 minutes.
- [ ] Different people applying the same framework to the same test case should reach the same conclusion.

## What You Should Understand After This Exercise

A good decision framework removes subjectivity from infrastructure choices. It forces teams to consider measurable factors (team size, service count, traffic volume, compliance) rather than opinions or hype. The framework should be simple enough to use regularly but thorough enough to catch important edge cases. Most importantly, it should make the right answer obvious for clear cases and provide structure for deciding borderline cases.
