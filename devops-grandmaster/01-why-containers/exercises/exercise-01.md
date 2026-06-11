# Exercise 01: The "It Works On My Machine" Autopsy

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Diagnose *why* an application works in one environment but fails in another.
This exercise trains you to identify the root causes of environment inconsistency --
the exact problem containers were invented to solve.

## Scenario

A developer sends you this message:

> "I built a Python web app on my MacBook. It works perfectly.
> I pushed the code to our Ubuntu production server and it crashes on startup.
> Here is what I see:"

```
$ python3 app.py
Traceback (most recent call last):
  File "app.py", line 3, in <module>
    from datetime import timezone
ModuleNotFoundError: No module named 'datetime.timezone'
```

The developer's machine:

```
OS:          macOS 14 (Sonoma)
Python:      3.11.4
pip packages: flask==3.0.0, requests==2.31.0
```

The production server:

```
OS:          Ubuntu 20.04 LTS
Python:      3.6.9  (system Python)
pip packages: flask==1.1.2
```

## Tasks

### Part A: Identify the Differences

List every difference between the developer's machine and the production server
that could contribute to the failure. Be specific -- do not just say "different OS."

<details>
<summary>Hint</summary>

Think about:
- Python version differences
- Package version differences
- How Python was installed (brew vs apt vs system)
- OS-level dependencies
- Module availability across Python versions

</details>

### Part B: Explain the Failure

Why does `from datetime import timezone` fail on the production server?
Research when the `timezone` class was added to Python's `datetime` module.

<details>
<summary>Hint</summary>

Look up which Python version introduced `datetime.timezone`.
Compare that to the Python version on the production server.

</details>

### Part C: List All the Fixes

Propose at least **three** different ways to fix this problem.
For each fix, explain:
- What you would change
- Whether it prevents the *next* "works on my machine" problem
- How much effort it requires

<details>
<summary>Hint</summary>

Consider approaches ranging from quick-and-dirty to robust:
1. A one-line change on the server
2. A configuration file that pins versions
3. A packaging approach that bundles everything together

</details>

### Part D: The Deeper Question

Even if you fix the Python version mismatch, list **three more** things that could
differ between a developer's macOS laptop and an Ubuntu production server
that would cause a different "works on my machine" failure in the future.

<details>
<summary>Hint</summary>

Think beyond Python itself:
- System libraries (openssl, libffi, etc.)
- File system paths
- Environment variables
- Networking configuration
- User permissions

</details>

## Success Criteria

- [ ] You can identify at least 4 differences between the two environments
- [ ] You can explain why `datetime.timezone` fails specifically
- [ ] You propose at least 3 fixes, each with trade-off analysis
- [ ] You can list 3 additional potential environment mismatches
- [ ] You understand why containers address this entire class of problems

## What You Should Understand After This Exercise

The "it works on my machine" problem is not one problem -- it is a *category*
of problems caused by environment state leaking into application behavior.
Fixing one instance does not prevent the next one.
