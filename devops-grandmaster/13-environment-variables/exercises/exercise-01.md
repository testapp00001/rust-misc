# Exercise 01: The 12-Factor Config Philosophy

**Type:** Conceptual
**Time:** 15 minutes
**Difficulty:** Easy

## Objective

Understand why the 12-Factor App methodology treats configuration as a first-class concern, and why environment variables are the standard mechanism for externalizing that configuration. This exercise trains you to think about configuration design before writing code.

## Background

Your team is building a new microservice. Before writing any code, the tech lead asks everyone to answer a set of design questions about configuration. The goal is to agree on principles so the team does not end up with hardcoded values, environment-specific Dockerfiles, or secrets committed to git.

## Tasks

### Part A: Map the 12 Factors to Configuration

The 12-Factor App methodology (https://12factor.net/) defines twelve principles for building modern, cloud-native applications. Four of them directly address configuration.

List the four factors most relevant to environment variable configuration. For each one, write a one-sentence explanation of what it requires and why it matters for containerized applications.

<details>
<summary>Hint</summary>

The module README includes a table mapping factors III, V, VI, and X to environment variables. Think about which factors deal with *where* config lives, *when* it is applied, and *how* it differs between environments.

</details>

### Part B: Config Files vs. Environment Variables

A junior developer on your team suggests storing all configuration in a `config.json` file mounted into the container via a volume. They argue it is easier to edit a JSON file than to manage dozens of environment variables.

Write a short response (3-5 sentences) explaining why the 12-Factor methodology prefers environment variables over config files for containerized applications. Address at least two specific drawbacks of the file-based approach.

<details>
<summary>Hint</summary>

Consider: What happens when you need different config for dev vs. prod? What happens if the config file has a syntax error? How does a file-based approach interact with immutable images?

</details>

### Part C: Classify Configuration Values

You are deploying a web application. Below is a list of configuration values. Classify each as either **config** (should be an environment variable) or **secret** (needs a secrets manager, covered in Module 14).

1. `DATABASE_URL` -- connection string with embedded password
2. `LOG_LEVEL` -- one of debug, info, warn, error
3. `API_KEY` -- third-party payment processor key
4. `PORT` -- the port the app listens on
5. `JWT_SECRET` -- signing key for authentication tokens
6. `NODE_ENV` -- production or development
7. `SMTP_PASSWORD` -- email service password
8. `MAX_UPLOAD_SIZE` -- file upload limit in MB

<details>
<summary>Hint</summary>

Config values are non-sensitive settings that control application behavior. Secrets are credentials, keys, or tokens that, if exposed, would compromise security. Environment variables are appropriate for config but not ideal for secrets because they are visible via `docker inspect`.

</details>

### Part D: The Configuration Hierarchy

The module describes a configuration hierarchy where higher-priority sources override lower-priority ones. Draw or describe this hierarchy with all six levels, from highest to lowest priority. For each level, give one example of when you would use it.

<details>
<summary>Hint</summary>

The hierarchy goes from runtime overrides (most specific) down to code defaults (most generic). Think about who sets each level: the developer, the CI/CD pipeline, the platform team, or the application author.

</details>

## Success Criteria

- [ ] You identified all four 12-Factor factors relevant to configuration (III, V, VI, X).
- [ ] You can explain why environment variables beat config files for containerized apps.
- [ ] You correctly classified all eight values as config or secret.
- [ ] You listed the full six-level configuration hierarchy with examples.
- [ ] You can articulate the difference between build-time, deploy-time, and runtime configuration.

## What You Should Understand After This Exercise

Configuration is not an afterthought -- it is a design decision. The 12-Factor methodology exists because mixing config with code leads to brittle, environment-specific artifacts. Environment variables are the standard because they are language-agnostic, require no file parsing, and are injected at runtime by the orchestrator. Understanding this philosophy is the foundation for every exercise that follows.
