# Exercise 01: What Happens When You Run a Container?

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand every step that Docker performs when you type `docker run`.
This exercise builds a mental model of the container lifecycle from the moment
you press Enter to the moment the container exits.

## Scenario

A new teammate joins your project. They have never used Docker before.
They type the following command and see the output, but have no idea what
actually happened behind the scenes:

```bash
$ docker run hello-world
Unable to find image 'hello-world:latest' locally
latest: Pulling from library/hello-world
c1ec31eb5944: Pull complete
Digest: sha256:...
Status: Downloaded newer image for hello-world:latest

Hello from Docker!
This message shows that your installation appears to be working correctly.
...
```

They turn to you and ask: "What just happened?"

## Tasks

### Part A: Break Down the Command

The command `docker run hello-world` looks simple, but Docker performs at
least **six distinct steps** when executing it. List and explain each step
in your own words.

<details>
<summary>Hint</summary>

Think about what Docker must do *before* it can run anything:
- Where does the image come from?
- What is an image vs a container?
- How does the container actually execute?

</details>

### Part B: The Full Command Anatomy

The command `docker run hello-world` uses defaults for many options.
Rewrite the command with **every implicit option made explicit**.
For example, `docker run` implicitly uses the `latest` tag -- so one
explicit version would be `docker run hello-world:latest`.

List at least **four** implicit options that Docker fills in for you.

<details>
<summary>Hint</summary>

Consider these dimensions:
- Image tag (what version?)
- Container naming
- Network mode
- Whether STDIN is connected
- What happens to the container after it exits

</details>

### Part C: Image vs Container

Explain the difference between an **image** and a **container** using an
analogy that a non-technical person would understand. Then answer:

1. How many containers can you create from one image?
2. If you run `docker run hello-world` twice, how many containers exist afterward?
3. If you run `docker run hello-world` twice, how many images exist on disk?

<details>
<summary>Hint</summary>

Think about a class vs an instance in programming, or a blueprint vs a house.
After running the command twice, check with `docker ps -a` and `docker images`.

</details>

### Part D: What If the Image Already Exists?

Imagine the `hello-world` image is already downloaded on your machine.
Describe what changes in the execution flow. Which steps from Part A are
skipped? Which still happen?

<details>
<summary>Hint</summary>

Docker caches images locally. What does "Unable to find image ... locally"
tell you about when Docker decides to pull?

</details>

## Success Criteria

- [ ] You can list at least 6 steps Docker performs during `docker run`
- [ ] You can identify at least 4 implicit options in `docker run hello-world`
- [ ] You can clearly explain the image vs container distinction
- [ ] You can describe what happens differently when an image is already cached
- [ ] You understand that `docker run` = `docker create` + `docker start`

## What You Should Understand After This Exercise

`docker run` is not a single atomic operation. It is a composition of
multiple steps -- pulling, creating, starting -- each of which you can
also perform independently. Understanding this decomposition is the
foundation for debugging container problems later.
