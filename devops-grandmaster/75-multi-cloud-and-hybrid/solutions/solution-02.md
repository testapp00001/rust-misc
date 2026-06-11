# Solution 02: Abstract Cloud Services with Terraform

## Directory Structure

```
exercise-02/
  main.tf
  variables.tf
  outputs.tf
  locals.tf
  modules/
    compute/
      aws/main.tf, variables.tf, outputs.tf
      azure/main.tf, variables.tf, outputs.tf
      gcp/main.tf, variables.tf, outputs.tf
    storage/
      aws/main.tf, variables.tf, outputs.tf
      azure/main.tf, variables.tf, outputs.tf
      gcp/main.tf, variables.tf, outputs.tf
    database/
      aws/main.tf, variables.tf, outputs.tf
      azure/main.tf, variables.tf, outputs.tf
      gcp/main.tf, variables.tf, outputs.tf
```

## Root Module

### variables.tf

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
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "environment must be dev, staging, or prod."
  }
}

variable "region" {
  type        = string
  description = "Cloud region for deployment"
}

variable "project_name" {
  type        = string
  description = "Name of the project, used for resource naming"
}

variable "instance_size" {
  type        = string
  description = "Common instance size: small, medium, or large"
  default     = "small"
}

variable "db_size" {
  type        = string
  description = "Database size: small, medium, or large"
  default     = "small"
}
```

### locals.tf

```hcl
locals {
  instance_sizes = {
    aws = {
      small  = "t3.micro"
      medium = "t3.medium"
      large  = "t3.large"
    }
    azure = {
      small  = "Standard_B1s"
      medium = "Standard_B2s"
      large  = "Standard_B4s"
    }
    gcp = {
      small  = "e2-micro"
      medium = "e2-medium"
      large  = "e2-standard-4"
    }
  }

  db_sizes = {
    aws = {
      small  = "db.t3.micro"
      medium = "db.t3.medium"
      large  = "db.r5.large"
    }
    azure = {
      small  = "B_Standard_B1ms"
      medium = "GP_Standard_D2s_v3"
      large  = "GP_Standard_D4s_v3"
    }
    gcp = {
      small  = "db-f1-micro"
      medium = "db-custom-2-4096"
      large  = "db-custom-4-16384"
    }
  }

  resource_prefix = "${var.project_name}-${var.environment}"
}
```

### main.tf

```hcl
terraform {
  required_version = ">= 1.5.0"
}

# Provider blocks -- only the active one needs valid credentials
provider "aws" {
  region = var.region
  alias  = "primary"
}

provider "azurerm" {
  features {}
  alias = "primary"
}

provider "google" {
  region = var.region
  alias  = "primary"
}

# --- Compute ---
module "compute_aws" {
  count  = var.provider_name == "aws" ? 1 : 0
  source = "./modules/compute/aws"
  providers = { aws = aws.primary }
  instance_size = local.instance_sizes["aws"][var.instance_size]
  project_name  = local.resource_prefix
  environment   = var.environment
}

module "compute_azure" {
  count  = var.provider_name == "azure" ? 1 : 0
  source = "./modules/compute/azure"
  providers = { azurerm = azurerm.primary }
  instance_size = local.instance_sizes["azure"][var.instance_size]
  project_name  = local.resource_prefix
  environment   = var.environment
  region        = var.region
}

module "compute_gcp" {
  count  = var.provider_name == "gcp" ? 1 : 0
  source = "./modules/compute/gcp"
  providers = { google = google.primary }
  instance_size = local.instance_sizes["gcp"][var.instance_size]
  project_name  = local.resource_prefix
  environment   = var.environment
  region        = var.region
}

# --- Storage ---
module "storage_aws" {
  count  = var.provider_name == "aws" ? 1 : 0
  source = "./modules/storage/aws"
  providers = { aws = aws.primary }
  bucket_name       = "${local.resource_prefix}-storage"
  versioning_enabled = var.environment == "prod"
}

module "storage_azure" {
  count  = var.provider_name == "azure" ? 1 : 0
  source = "./modules/storage/azure"
  providers = { azurerm = azurerm.primary }
  bucket_name       = "${local.resource_prefix}-storage"
  versioning_enabled = var.environment == "prod"
  project_name      = local.resource_prefix
  region            = var.region
}

module "storage_gcp" {
  count  = var.provider_name == "gcp" ? 1 : 0
  source = "./modules/storage/gcp"
  providers = { google = google.primary }
  bucket_name       = "${local.resource_prefix}-storage"
  versioning_enabled = var.environment == "prod"
}

# --- Database ---
module "database_aws" {
  count  = var.provider_name == "aws" ? 1 : 0
  source = "./modules/database/aws"
  providers = { aws = aws.primary }
  db_name      = "appdb"
  instance_class = local.db_sizes["aws"][var.db_size]
  project_name = local.resource_prefix
  environment  = var.environment
}

module "database_azure" {
  count  = var.provider_name == "azure" ? 1 : 0
  source = "./modules/database/azure"
  providers = { azurerm = azurerm.primary }
  db_name       = "appdb"
  instance_class = local.db_sizes["azure"][var.db_size]
  project_name  = local.resource_prefix
  environment   = var.environment
  region        = var.region
}

module "database_gcp" {
  count  = var.provider_name == "gcp" ? 1 : 0
  source = "./modules/database/gcp"
  providers = { google = google.primary }
  db_name       = "appdb"
  instance_class = local.db_sizes["gcp"][var.db_size]
  project_name  = local.resource_prefix
  environment   = var.environment
  region        = var.region
}
```

### outputs.tf

```hcl
locals {
  active_compute  = var.provider_name == "aws" ? module.compute_aws : (
    var.provider_name == "azure" ? module.compute_azure : module.compute_gcp
  )
  active_storage  = var.provider_name == "aws" ? module.storage_aws : (
    var.provider_name == "azure" ? module.storage_azure : module.storage_gcp
  )
  active_database = var.provider_name == "aws" ? module.database_aws : (
    var.provider_name == "azure" ? module.database_azure : module.database_gcp
  )
}

output "instance_id" {
  value       = local.active_compute[0].instance_id
  description = "ID of the compute instance"
}

output "private_ip" {
  value       = local.active_compute[0].private_ip
  description = "Private IP of the compute instance"
}

output "storage_endpoint" {
  value       = local.active_storage[0].storage_endpoint
  description = "Endpoint for the storage bucket"
}

output "database_endpoint" {
  value       = local.active_database[0].database_endpoint
  description = "Connection endpoint for the database"
}
```

## Provider-Specific Modules (Examples)

### modules/compute/aws/main.tf

```hcl
terraform {
  required_providers {
    aws = { source = "hashicorp/aws" }
  }
}

data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"]
  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-22.04-amd64-server-*"]
  }
}

resource "aws_instance" "main" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = var.instance_size
  tags = {
    Name        = var.project_name
    Environment = var.environment
  }
}
```

### modules/compute/azure/main.tf

```hcl
terraform {
  required_providers {
    azurerm = { source = "hashicorp/azurerm" }
  }
}

resource "azurerm_resource_group" "main" {
  name     = "${var.project_name}-rg"
  location = var.region
}

resource "azurerm_virtual_network" "main" {
  name                = "${var.project_name}-vnet"
  address_space       = ["10.0.0.0/16"]
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
}

resource "azurerm_subnet" "internal" {
  name                 = "internal"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.2.0/24"]
}

resource "azurerm_network_interface" "main" {
  name                = "${var.project_name}-nic"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  ip_configuration {
    name                          = "internal"
    subnet_id                     = azurerm_subnet.internal.id
    private_ip_address_allocation = "Dynamic"
  }
}

resource "azurerm_linux_virtual_machine" "main" {
  name                = "${var.project_name}-vm"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  size                = var.instance_size
  admin_username      = "adminuser"
  network_interface_ids = [azurerm_network_interface.main.id]
  os_disk {
    caching              = "ReadWrite"
    storage_account_type = "Standard_LRS"
  }
  source_image_reference {
    publisher = "Canonical"
    offer     = "0001-com-ubuntu-server-jammy"
    sku       = "22_04-lts"
    version   = "latest"
  }
}
```

### modules/compute/gcp/main.tf

```hcl
terraform {
  required_providers {
    google = { source = "hashicorp/google" }
  }
}

resource "google_compute_instance" "main" {
  name         = "${var.project_name}-vm"
  machine_type = var.instance_size
  zone         = "${var.region}-a"
  boot_disk {
    initialize_params {
      image = "ubuntu-2204-lts"
    }
  }
  network_interface {
    network = "default"
    access_config {}
  }
  labels = {
    environment = var.environment
  }
}
```

## Key Design Decisions

1. **Conditional module invocation**: Using `count` with provider name
   comparison ensures only one provider's modules are active. This avoids
   requiring credentials for all three providers simultaneously.

2. **Size normalization**: The `locals.instance_sizes` map translates
   abstract sizes (small/medium/large) to provider-specific values. This
   keeps the root interface clean.

3. **Consistent outputs**: All modules return the same four outputs. The
   root module selects the active one using a ternary chain. This is
   admittedly verbose but works without dynamic module references.

4. **Provider aliasing**: Each provider is aliased to avoid conflicts. The
   unused providers still need to be configured but can use dummy values
   when not active.

## Limitations to Acknowledge

- Terraform loads ALL module blocks even if `count = 0`, so all providers
  must be configured (or use workspaces to isolate).
- Provider-specific features (AWS Spot Instances, Azure Proximity Placement
  Groups, GCP Preemptible VMs) cannot be exposed through the common
  interface without complicating it.
- State management becomes more complex when the same logical resource can
  be a different physical resource type depending on the provider.
