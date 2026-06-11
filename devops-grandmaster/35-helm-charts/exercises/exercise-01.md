# Exercise 01: Chart Structure and Templating (Conceptual)

## Objective

Understand the anatomy of a Helm chart, how Go templating works inside
Helm, and how `values.yaml` flows into templates to produce Kubernetes
manifests.

## Background

Helm is the package manager for Kubernetes. A Helm **chart** is a bundle
of templated Kubernetes manifests plus metadata. When you install a chart,
Helm renders the templates with your supplied values and applies the
resulting manifests to the cluster.

## Instructions

### Part A -- Explore an existing chart

1. Create a new chart called `demo`:

   ```bash
   helm create demo
   ```

2. List every file inside the `demo/` directory recursively. Draw (on
   paper or in a text file) the full directory tree.

3. For each of the following files, write a one-sentence description of
   its purpose:

   - `Chart.yaml`
   - `values.yaml`
   - `templates/deployment.yaml`
   - `templates/service.yaml`
   - `templates/_helpers.tpl`
   - `templates/NOTES.txt`
   - `.helmignore`

### Part B -- Trace the template pipeline

1. Open `templates/deployment.yaml` and identify every `{{ .Values.* }}`
   expression. List each one and the path it reads from `values.yaml`.

2. Run a dry-run render so you can see the final output:

   ```bash
   helm template myrelease demo/
   ```

3. Change the value of `replicaCount` in `values.yaml` from `1` to `3`,
   re-render, and confirm the change appears in the output.

### Part C -- Template functions and pipelines

1. In `templates/deployment.yaml`, find the label block. Explain what
   `include` does versus `template`.

2. Open `templates/_helpers.tpl`. Identify the `define` blocks and explain
   how they create reusable template snippets.

3. Add a new helper in `_helpers.tpl` called `demo.fullname` that produces
   `<release-name>-demo`. Use it in `deployment.yaml` for the `metadata.name`
   field. Re-render to verify.

### Part D -- Built-in objects

Write a short answer (2-3 sentences each) for the following:

1. What is `.Release` and what useful properties does it contain?
2. What is `.Chart` and when would you reference it?
3. What is the difference between `.Values` and `.Capabilities`?

## Success Criteria

- [ ] You can describe the purpose of every file `helm create` generates.
- [ ] You can list the template expressions in `deployment.yaml` and trace
      each one to its `values.yaml` source.
- [ ] You can explain the difference between `include` and `template`.
- [ ] You can describe `.Release`, `.Chart`, and `.Capabilities`.
- [ ] You have rendered a chart with `helm template` and observed how
      value changes propagate to output.

## Hints

<details>
<summary>Hint: File purposes</summary>

- `Chart.yaml` -- chart metadata (name, version, apiVersion, dependencies).
- `values.yaml` -- default configuration values.
- `templates/` -- Kubernetes manifest templates with Go template syntax.
- `_helpers.tpl` -- reusable template definitions (partials).
- `NOTES.txt` -- post-install message displayed to the user.
- `.helmignore` -- patterns to exclude when packaging.

</details>

<details>
<summary>Hint: include vs template</summary>

`template` inserts a named template inline. `include` does the same but
returns the result as a string, which means you can pipe it through other
functions (e.g., `include "demo.name" . | quote`).

</details>

<details>
<summary>Hint: Built-in objects</summary>

- `.Release.Name` -- the release name you pass with `helm install myrelease ...`
- `.Release.Namespace` -- the target namespace.
- `.Chart.Version` -- version from `Chart.yaml`.
- `.Capabilities.KubeVersion` -- the Kubernetes server version.

</details>
