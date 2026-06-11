# Exercise 04: Shrink a 1.2GB Java Image to Under 200MB

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Take an existing single-stage Java Dockerfile that produces a 1.2GB image and
reduce it to under 200MB using multi-stage builds. This exercise challenges you
to apply what you learned to a language with a large runtime ecosystem and
explore advanced techniques like `jlink` for custom JVM runtimes.

## Starting Point

You are given a Spring Boot-style Java application with this single-stage
Dockerfile:

```dockerfile
FROM maven:3.9-eclipse-temurin-21

WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline

COPY src ./src
RUN mvn package -DskipTests

EXPOSE 8080
ENTRYPOINT ["java", "-jar", "target/app.jar"]
```

This produces a 1.2GB image. The breakdown:

```
maven:3.9-eclipse-temurin-21 base:     ~800MB
  - JDK (compiler, tools, debug symbols): ~300MB
  - Maven (build tool):                   ~200MB
  - OS packages and utilities:            ~300MB
Maven dependencies (cached):              ~200MB
Application JAR:                           ~20MB
---------------------------------------------
Total:                                  ~1,200MB
```

## Application Code

Create the project structure:

```bash
mkdir -p /tmp/exercise-04/src/main/java/com/example
```

Create `/tmp/exercise-04/pom.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>shrink-demo</artifactId>
    <version>1.0</version>
    <properties>
        <maven.compiler.source>21</maven.compiler.source>
        <maven.compiler.target>21</maven.compiler.target>
    </properties>
</project>
```

Create `/tmp/exercise-04/src/main/java/com/example/App.java`:

```java
package com.example;

import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.io.OutputStream;
import java.net.InetSocketAddress;

public class App {
    public static void main(String[] args) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress(8080), 0);
        server.createContext("/", exchange -> {
            String response = "Hello from a Java container!\n";
            exchange.sendResponseHeaders(200, response.length());
            OutputStream os = exchange.getResponseBody();
            os.write(response.getBytes());
            os.close();
        });
        System.out.println("Server starting on :8080");
        server.start();
    }
}
```

## Tasks

### Part A: Basic Multi-Stage (Target: Under 400MB)

Convert the single-stage Dockerfile to a multi-stage Dockerfile with:
- A **builder** stage that uses `maven:3.9-eclipse-temurin-21` to compile
- A **runtime** stage that uses `eclipse-temurin:21-jre` (JRE only, no JDK)

This should get you under 400MB.

<details>
<summary>Hint</summary>

The JRE is much smaller than the JDK. It has the Java runtime to execute
bytecode but not the compiler or development tools.

```dockerfile
FROM maven:3.9-eclipse-temurin-21 AS builder
WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline
COPY src ./src
RUN mvn package -DskipTests

FROM eclipse-temurin:21-jre
WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
```

</details>

Build and measure:

```bash
cd /tmp/exercise-04
docker build -t exercise4-basic .
docker images exercise4-basic
```

### Part B: Add Security Best Practices (Still Under 400MB)

Improve the runtime stage by:
- Running as a non-root user
- Using a specific base image tag (not just `21-jre`)
- Adding a health check

<details>
<summary>Hint</summary>

```dockerfile
FROM eclipse-temurin:21-jre-jammy

RUN useradd --create-home --no-log-init appuser
WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar
USER appuser

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:8080/ || exit 1
ENTRYPOINT ["java", "-jar", "app.jar"]
```

Note: You may need to install `curl` in the runtime image for the
health check, or use a different approach.

</details>

### Part C: Custom JVM with jlink (Target: Under 200MB)

This is the hard part. Use `jlink` to create a custom JVM that contains
only the modules your application needs. This can reduce the JRE from
~200MB to ~40-60MB.

Your application uses `com.sun.net.httpserver`, which requires the
`java.net.http` module. Use `jdeps` or the module documentation to
figure out which modules to include.

The `jlink` command:

```bash
jlink \
    --add-modules <MODULES> \
    --strip-debug \
    --no-man-pages \
    --no-header-files \
    --compress=zip-6 \
    --output /jvm
```

Create a three-stage Dockerfile:
1. **builder** -- compiles the application with Maven
2. **jlink** -- creates a custom minimal JVM
3. **runtime** -- combines the custom JVM with the application JAR

<details>
<summary>Hint 1: Which Modules</summary>

The `com.sun.net.httpserver` package is part of the `jdk.httpserver`
module. The `java.base` module is always required. Use `jdeps` to
analyze your JAR:

```bash
jdeps --print-module-deps target/app.jar
```

Or start with `java.base,jdk.httpserver` and add others if the
application fails to start.

</details>

<details>
<summary>Hint 2: Three-Stage Dockerfile</summary>

```dockerfile
# Stage 1: Build
FROM maven:3.9-eclipse-temurin-21 AS builder
WORKDIR /app
COPY pom.xml .
RUN mvn dependency:go-offline
COPY src ./src
RUN mvn package -DskipTests

# Stage 2: Create custom JVM
FROM eclipse-temurin:21 AS jlink
RUN jlink \
    --add-modules java.base,jdk.httpserver \
    --strip-debug \
    --no-man-pages \
    --no-header-files \
    --compress=zip-6 \
    --output /jvm

# Stage 3: Runtime
FROM debian:bookworm-slim
COPY --from=jlink /jvm /opt/jvm
ENV PATH="/opt/jvm/bin:${PATH}"
WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar
RUN useradd --create-home appuser
USER appuser
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
```

</details>

### Part D: Measure and Compare

Build all three versions and compare:

```bash
docker build --target exercise4-basic -t exercise4-basic .
docker build -t exercise4-jlink .
docker images | grep exercise4
```

Fill in this table:

| Approach | Image Size | Reduction |
|---|---|---|
| Original (single-stage) | 1,200MB | baseline |
| Multi-stage with JRE | ??? MB | ???% |
| Multi-stage with jlink | ??? MB | ???% |

### Part E: Verify the jlink Image

The custom JVM is minimal. Verify your application still works:

```bash
docker run -p 8080:8080 exercise4-jlink &
curl http://localhost:8080
```

If it fails, check the error message. Common issues:
- Missing Java module (add it to `--add-modules`)
- Missing runtime library in `debian:bookworm-slim`

## Success Criteria

- [ ] Basic multi-stage image is under 400MB
- [ ] jlink image is under 200MB
- [ ] Runtime stage runs as a non-root user
- [ ] Application responds to HTTP requests in all variants
- [ ] You can explain what `jlink` does and when to use it
- [ ] You can name at least three Java modules and what they provide

## What You Should Understand After This Exercise

Java applications can be shrunk dramatically using multi-stage builds.
The first step is replacing the JDK with a JRE in the runtime stage.
The advanced step is using `jlink` to create a custom JVM with only
the modules your application needs. This trades flexibility (you
cannot run arbitrary Java programs) for size (the JVM is 40-60MB
instead of 200MB). For production microservices that do one thing,
this trade-off is almost always worth it.
