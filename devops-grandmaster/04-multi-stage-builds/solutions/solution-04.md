# Solution 04: Shrink a 1.2GB Java Image to Under 200MB

## Part A: Basic Multi-Stage (Under 400MB)

### Dockerfile

```dockerfile
# ===== Stage 1: Builder =====
FROM maven:3.9-eclipse-temurin-21 AS builder

WORKDIR /app

# Copy pom.xml first for dependency caching
COPY pom.xml .
RUN mvn dependency:go-offline

# Copy source and build
COPY src ./src
RUN mvn package -DskipTests

# ===== Stage 2: Runtime =====
FROM eclipse-temurin:21-jre

WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar

EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
```

**Why this reduces size from 1.2GB to ~350MB:**

The original image uses `maven:3.9-eclipse-temurin-21`, which contains:
- Full JDK (compiler, javadoc, debug symbols): ~300MB
- Maven build tool: ~200MB
- Full Debian OS with utilities: ~300MB
- Application JAR: ~20MB

The multi-stage image uses `eclipse-temurin:21-jre` for runtime, which contains:
- JRE only (runtime, no compiler): ~200MB
- Minimal OS: ~100MB
- Application JAR: ~20MB

The JDK, Maven, build caches, and source code are all left behind in the builder stage.

**What `mvn dependency:go-offline` does:**
- Downloads all project dependencies and plugins.
- Caches them in the local Maven repository (`~/.m2`).
- Makes subsequent `mvn package` builds faster and not require network access.
- Placed before `COPY src` so that source code changes do not invalidate the dependency cache.

**What `-DskipTests` does:**
- Skips running unit tests during the build.
- In a Docker build, tests should run in a separate CI step, not during image creation.
- Saves build time inside the Dockerfile.

**Expected size: ~350MB**

---

## Part B: Security Best Practices (Still Under 400MB)

### Dockerfile

```dockerfile
# ===== Stage 1: Builder =====
FROM maven:3.9-eclipse-temurin-21 AS builder

WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline
COPY src ./src
RUN mvn package -DskipTests

# ===== Stage 2: Runtime =====
FROM eclipse-temurin:21-jre-jammy

# Install curl for health checks
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --no-log-init appuser

WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar

# Switch to non-root
USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
    CMD curl -f http://localhost:8080/ || exit 1

ENTRYPOINT ["java", "-jar", "app.jar"]
```

**Improvements over Part A:**

1. **Non-root user:** `useradd --create-home --no-log-init appuser` creates a dedicated user. `USER appuser` switches to it. If an attacker exploits a vulnerability in the Java application, they get an unprivileged user instead of root.

2. **Pinned base image tag:** `eclipse-temurin:21-jre-jammy` specifies both the Java version (21) and the Ubuntu version (Jammy 22.04). This makes builds reproducible. Using just `21-jre` could resolve to a different Ubuntu version over time.

3. **Health check:** Docker (and orchestrators like Docker Compose and Swarm) can monitor container health and restart unhealthy containers. The `curl -f` command fails on non-2xx responses, triggering a health check failure.

4. **`--no-log-init`:** Prevents `useradd` from creating a lastlog/faillog entry, which can cause issues in some container environments.

**Note on `curl` installation:** The `eclipse-temurin:21-jre-jammy` image does not include `curl` by default. An alternative is to use a Java-based health check (e.g., a small HTTP client in the application itself) to avoid installing curl. Another option is to use `wget` if available.

**Expected size: ~360MB** (slightly larger than Part A due to curl)

---

## Part C: Custom JVM with jlink (Under 200MB)

### Dockerfile

```dockerfile
# ===== Stage 1: Build =====
FROM maven:3.9-eclipse-temurin-21 AS builder

WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline
COPY src ./src
RUN mvn package -DskipTests

# ===== Stage 2: Create Custom JVM =====
FROM eclipse-temurin:21 AS jlink

RUN jlink \
    --add-modules java.base,jdk.httpserver \
    --strip-debug \
    --no-man-pages \
    --no-header-files \
    --compress=zip-6 \
    --output /jvm

# ===== Stage 3: Runtime =====
FROM debian:bookworm-slim

# Copy custom JVM
COPY --from=jlink /jvm /opt/jvm
ENV PATH="/opt/jvm/bin:${PATH}"

# Create non-root user
RUN useradd --create-home appuser

WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar

USER appuser
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
```

**How jlink works:**

The standard JRE is a general-purpose runtime that includes every Java module. Your application only uses a few of them. `jlink` creates a custom JVM image containing only the modules you specify.

**Module selection:**

The application uses `com.sun.net.httpserver.HttpServer`, which is part of the `jdk.httpserver` module. The `java.base` module is always required -- it contains the core classes (java.lang, java.io, java.net, etc.).

To find required modules, run `jdeps` on the JAR:

```bash
jdeps --print-module-deps target/app.jar
```

For this application, the output would be:
```
java.base
jdk.httpserver
```

**jlink flags explained:**
- `--add-modules java.base,jdk.httpserver`: Include only these modules.
- `--strip-debug`: Remove debug symbols from the JVM libraries.
- `--no-man-pages`: Exclude man pages.
- `--no-header-files`: Exclude C header files (for JNI development).
- `--compress=zip-6`: Maximum compression level for JVM resources.
- `--output /jvm`: Where to write the custom JVM.

**Why three stages instead of two:**

You could combine jlink into the builder stage, but separating them has advantages:
- The jlink stage only depends on the JDK, not on Maven or your source code.
- If you change source code but not the module list, the jlink stage is cached.
- It makes the Dockerfile easier to read and maintain.

**Why `debian:bookworm-slim` instead of scratch:**

Unlike Go, Java cannot produce a fully static binary. The JVM needs a libc and a Linux userspace. `debian:bookworm-slim` provides just enough OS for the JVM to run (~80MB). You could also use Alpine, but the JVM may have compatibility issues with musl libc.

**Expected size: ~150-180MB**

---

## Part D: Size Comparison

| Approach | Image Size | Reduction |
|---|---|---|
| Original (single-stage `maven:3.9-eclipse-temurin-21`) | ~1,200MB | baseline |
| Multi-stage with `eclipse-temurin:21-jre` | ~350MB | ~71% |
| Multi-stage with jlink custom JVM | ~160MB | ~87% |

The jlink approach is roughly half the size of the JRE approach because:
- Full JRE: ~200MB (contains all Java modules)
- Custom JVM: ~40-60MB (contains only `java.base` + `jdk.httpserver`)
- The rest of the image (Debian slim + JAR) is the same in both cases.

---

## Part E: Verify the jlink Image

```bash
# Build the jlink image
docker build -t exercise4-jlink .

# Run it
docker run -p 8080:8080 exercise4-jlink &

# Test it
curl http://localhost:8080
# Expected: "Hello from a Java container!"
```

**Troubleshooting:**

If the application fails to start with `NoClassDefFoundError`:
- Check which class is missing.
- Find which Java module contains that class (use the JDK module documentation or `jdeps`).
- Add that module to `--add-modules`.

If the application fails with `UnsatisfiedLinkError`:
- A native library is missing. This is rare for pure Java applications.
- May need to add `jdk.unsupported` for legacy APIs.

If `debian:bookworm-slim` is missing a required library:
- The JVM needs libc, libz, and libpthread, which are included in Debian slim.
- If you see `libjli.so: cannot open shared object file`, the PATH may be wrong.

---

## Common Mistakes

1. **Using the JDK in the runtime stage.** The JDK includes the compiler (javac), debugger (jdb), and documentation tools. None of these are needed at runtime. Use the JRE or jlink.

2. **Not specifying `--add-modules` for jlink.** If you omit this flag, jlink includes all modules, which defeats the purpose. The whole point is to include only what your application needs.

3. **Forgetting `java.base` in `--add-modules`.** The `java.base` module is required for any Java application. Omitting it causes the JVM to fail immediately.

4. **Not copying CA certificates.** If your Java application makes HTTPS calls (common for microservices), the `debian:bookworm-slim` base may not have CA certificates. Add `RUN apt-get update && apt-get install -y ca-certificates` or copy them from the builder.

5. **Using `FROM scratch` for Java.** Java cannot produce a fully static binary. The JVM always needs a libc and OS. `FROM scratch` will not work for Java applications.

6. **Not using `mvn dependency:go-offline`.** Without this step, `mvn package` downloads dependencies on every build. If the network is slow or unavailable, the build fails. `go-offline` pre-downloads everything.

7. **Putting `USER appuser` before `COPY --from=builder`.** The `COPY` instruction copies files as root. If you switch to a non-root user first, the copied files may be owned by root but the process cannot read them (depending on permissions). Copy files first, then switch user.

8. **Using `ENTRYPOINT` and `CMD` incorrectly.** `ENTRYPOINT ["java", "-jar", "app.jar"]` is correct. If you used `CMD java -jar app.jar` (shell form), the Java process would not be PID 1 and would not receive signals properly.
