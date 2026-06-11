# Exercise 02: Abstract Cloud Services with Terraform

**Type:** Guided
**Difficulty:** Intermediate
**Estimated Time:** 2-3 hours

## Objective

Build a Terraform module that abstracts common cloud resources (compute
instances, object storage, managed databases) behind a provider-agnostic
interface. The same module configuration should be deployable to AWS, Azure,
or GCP by changing only the provider variable.

## Background

Vendor lock-in often starts at the infrastructure-as-code level. When your
Terraform configurations directly reference `aws_instance` or
`azurerm_virtual_machine`, migrating to another provider requires rewriting
every resource. By creating abstraction layers, you can deploy the same
logical infrastructure to multiple providers.

This exercise uses Terraform workspaces and provider-specific submodules to
achieve abstraction without sacrificing access to provider-native features.

## Instructions

### Step 1: Create the Directory Structure

```
exercise-02/
  main.tf
  variables.tf
  outputs.tf
  modules/
    compute/
      aws/
        main.tf
        variables.tf
        outputs.tf
      azure/
        main.tf
        variables.tf
        outputs.tf
      gcp/
        main.tf
        variables.tf
        outputs.tf
    storage/
      aws/
        main.tf
        variables.tf
        outputs.tf
      azure/
        main.tf
        variables.tf
        outputs.tf
      gcp/
        main.tf
        variables.tf
        outputs.tf
    database/
      aws/
        main.tf
        variables.tf
        outputs.tf
      azure/
        main.tf
        variables.tf
        outputs.tf
      gcp/
        main.tf
        variables.tf
        outputs.tf
```

### Step 2: Define the Common Interface

In the root `variables.tf`, define a `provider_name` variable and common
variables that all providers must accept:

```hcl
variable "provider_name" {
  type        = string
  description = "Target cloud provider: aws, azure, or gcp"
  validation {
    condition     = contains(["aws", "azure", "gcp"], var.provider_name)
    error_message = "provider_name must be aws, azure, or gcp."
  }
}

variable "environment" {
  type        = string
  description = "Deployment environment (dev, staging, prod)"
}

variable "region" {
  type        = string
  description = "Cloud region for deployment"
}

variable "project_name" {
  type        = string
  description = "Name of the project, used for resource naming"
}
```

### Step 3: Implement Provider Routing

In the root `main.tf`, use `count` or `for_each` to conditionally call the
correct provider-specific submodule:

```hcl
module "compute" {
  source = "./modules/compute/${var.provider_name}"
  # Pass through common variables
}

module "storage" {
  source = "./modules/storage/${var.provider_name}"
  # Pass through common variables
}

module "database" {
  source = "./modules/database/${var.provider_name}"
  # Pass through common variables
}
```

### Step 4: Implement Each Provider Module

For each resource type, implement the provider-specific version:

**Compute** -- Each provider module should create:
- AWS: `aws_instance` with a specified AMI
- Azure: `azurerm_linux_virtual_machine`
- GCP: `google_compute_instance`

All should accept the same variables: `instance_size`, `image_id`, `subnet_id`.

**Storage** -- Each provider module should create:
- AWS: `aws_s3_bucket`
- Azure: `azurerm_storage_container`
- GCP: `google_storage_bucket`

All should accept: `bucket_name`, `versioning_enabled`, `encryption_key`.

**Database** -- Each provider module should create:
- AWS: `aws_db_instance` (RDS PostgreSQL)
- Azure: `azurerm_postgresql_flexible_server`
- GCP: `google_sql_database_instance` (PostgreSQL)

All should accept: `db_name`, `db_size`, `instance_class`, `engine_version`.

### Step 5: Normalize Outputs

Each provider module must return outputs in a consistent format:

```hcl
output "instance_id" {
  value = <provider-specific ID>
}

output "private_ip" {
  value = <provider-specific IP>
}

output "storage_endpoint" {
  value = <provider-specific endpoint>
}

output "database_endpoint" {
  value = <provider-specific connection string>
}
```

### Step 6: Test Provider Switching

Verify that by changing only `provider_name` from `"aws"` to `"azure"` to
`"gcp"`, the plan shows the correct provider-specific resources being created
while the logical structure remains identical.

## Success Criteria

- [ ] The root module accepts `provider_name` and routes to the correct
      submodule without errors.
- [ ] All three provider implementations (AWS, Azure, GCP) exist for compute,
      storage, and database (9 submodules total).
- [ ] Each submodule uses the same input variable names so the root module
      passes identical arguments regardless of provider.
- [ ] Each submodule normalizes outputs to the same schema.
- [ ] `terraform validate` passes for all three provider configurations.
- [ ] You can explain which provider features are lost by using this
      abstraction and when that trade-off is acceptable.

## Hints

<details>
<summary>Hint 1: Dynamic source paths</summary>

Terraform does not support variable interpolation in `source` paths. Use
`count` with conditional logic instead:

```hcl
module "compute_aws" {
  count  = var.provider_name == "aws" ? 1 : 0
  source = "./modules/compute/aws"
}

module "compute_azure" {
  count  = var.provider_name == "azure" ? 1 : 0
  source = "./modules/compute/azure"
}
```

Then reference outputs via `module.compute_aws[0].instance_id` with a
`locals` block to pick the active one.

</details>

<details>
<summary>Hint 2: Normalizing instance sizes</summary>

Cloud providers use different instance size naming conventions. Create a
locals map to translate a common size name to provider-specific values:

```hcl
locals {
  instance_sizes = {
    aws   = { small = "t3.micro", medium = "t3.medium", large = "t3.large" }
    azure = { small = "Standard_B1s", medium = "Standard_B2s", large = "Standard_B4s" }
    gcp   = { small = "e2-micro", medium = "e2-medium", large = "e2-standard-4" }
  }
}
```

</details>

<details>
<summary>Hint 3: Handling provider blocks</summary>

You need provider blocks for all three providers even if you only use one at
a time. Use `alias` and make each provider optional by passing an empty
configuration or using a variable to control which providers are actually
configured.

</details>
