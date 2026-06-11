# Solution 03: Implement Cross-Cloud Networking

## Network Architecture

```
   AWS VPC (10.1.0.0/16)                    Azure VNet (10.2.0.0/16)
  +------------------------+              +-------------------------+
  | Public Subnet          |              | Subnet 1                |
  | 10.1.1.0/24            |              | 10.2.1.0/24             |
  |                        |              |                         |
  | Private Subnet         |   VPN Tunnel | Subnet 2                |
  | 10.1.2.0/24            |<============>| 10.2.2.0/24             |
  |   EC2: 10.1.2.10       |   IPsec      |   VM: 10.2.2.10         |
  |                        |              |                         |
  | VGW <------------------+---tunnel-----+-> VPN Gateway           |
  +------------------------+              | 10.2.3.0/27 (GW Sub)   |
                                          +-------------------------+

  Route tables:
  AWS VPC:   10.2.0.0/16 -> VGW
  Azure VNet: 10.1.0.0/16 -> VPN Gateway
```

## AWS Side -- aws-network.tf

```hcl
# Variables
variable "aws_region" {
  default = "us-east-1"
}

variable "aws_vpc_cidr" {
  default = "10.1.0.0/16"
}

# VPC
resource "aws_vpc" "main" {
  cidr_block           = var.aws_vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true
  tags = { Name = "cross-cloud-vpc" }
}

# Subnets
resource "aws_subnet" "public" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.1.1.0/24"
  availability_zone = "${var.aws_region}a"
  tags = { Name = "public-subnet" }
}

resource "aws_subnet" "private" {
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.1.2.0/24"
  availability_zone = "${var.aws_region}a"
  tags = { Name = "private-subnet" }
}

# Internet Gateway
resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "cross-cloud-igw" }
}

# Virtual Private Gateway
resource "aws_vpn_gateway" "main" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "cross-cloud-vgw" }
}

# Customer Gateway (Azure VPN Gateway public IP)
variable "azure_vpn_gateway_ip" {
  description = "Public IP of the Azure VPN Gateway"
  type        = string
}

resource "aws_customer_gateway" "azure" {
  bgp_asn    = 65000
  ip_address = var.azure_vpn_gateway_ip
  type       = "ipsec.1"
  tags       = { Name = "azure-customer-gateway" }
}

# VPN Connection
resource "aws_vpn_connection" "to_azure" {
  vpn_gateway_id      = aws_vpn_gateway.main.id
  customer_gateway_id = aws_customer_gateway.azure.id
  type                = "ipsec.1"
  static_routes_only  = true
  tags                = { Name = "aws-to-azure-vpn" }
}

# Static route for Azure traffic
resource "aws_vpn_connection_route" "to_azure" {
  destination_cidr_block = "10.2.0.0/16"
  vpn_connection_id      = aws_vpn_connection.to_azure.id
}

# Route table for private subnet
resource "aws_route_table" "private" {
  vpc_id = aws_vpc.main.id
  tags   = { Name = "private-rt" }
}

resource "aws_route" "to_azure" {
  route_table_id         = aws_route_table.private.id
  destination_cidr_block = "10.2.0.0/16"
  gateway_id             = aws_vpn_gateway.main.id
}

resource "aws_route_table_association" "private" {
  subnet_id      = aws_subnet.private.id
  route_table_id = aws_route_table.private.id
}

# Security Group for cross-cloud traffic
resource "aws_security_group" "cross_cloud" {
  name        = "cross-cloud-sg"
  description = "Allow traffic from Azure VNet"
  vpc_id      = aws_vpc.main.id

  ingress {
    description = "ICMP from Azure"
    from_port   = -1
    to_port     = -1
    protocol    = "icmp"
    cidr_blocks = ["10.2.0.0/16"]
  }

  ingress {
    description = "SSH from Azure"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["10.2.0.0/16"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# Test instance in private subnet
resource "aws_instance" "test" {
  ami                    = "ami-0c55b159cbfafe1f0"  # Ubuntu 22.04 us-east-1
  instance_type          = "t3.micro"
  subnet_id              = aws_subnet.private.id
  vpc_security_group_ids = [aws_security_group.cross_cloud.id]
  tags                   = { Name = "aws-test-instance" }
}

output "aws_instance_private_ip" {
  value = aws_instance.test.private_ip
}

output "aws_vpn_tunnel1_address" {
  value = aws_vpn_connection.to_azure.tunnel1_address
}
```

## Azure Side -- azure-network.tf

```hcl
variable "azure_region" {
  default = "East US"
}

variable "azure_vnet_cidr" {
  default = "10.2.0.0/16"
}

# Resource Group
resource "azurerm_resource_group" "main" {
  name     = "cross-cloud-rg"
  location = var.azure_region
}

# Virtual Network
resource "azurerm_virtual_network" "main" {
  name                = "cross-cloud-vnet"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  address_space       = [var.azure_vnet_cidr]
}

# Subnets
resource "azurerm_subnet" "subnet1" {
  name                 = "subnet-1"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.2.1.0/24"]
}

resource "azurerm_subnet" "subnet2" {
  name                 = "subnet-2"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.2.2.0/24"]
}

# GatewaySubnet (required, exact name)
resource "azurerm_subnet" "gateway" {
  name                 = "GatewaySubnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.2.3.0/27"]
}

# Public IP for VPN Gateway
resource "azurerm_public_ip" "vpn" {
  name                = "vpn-gateway-ip"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  allocation_method   = "Static"
  sku                 = "Standard"
}

# VPN Gateway (takes 20-45 minutes to provision)
resource "azurerm_virtual_network_gateway" "vpn" {
  name                = "cross-cloud-vpn-gw"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  type                = "Vpn"
  vpn_type            = "RouteBased"
  active_active       = false
  enable_bgp          = false
  sku                 = "VpnGw1"

  ip_configuration {
    name                          = "vnetGatewayConfig"
    public_ip_address_id          = azurerm_public_ip.vpn.id
    private_ip_address_allocation = "Dynamic"
    subnet_id                     = azurerm_subnet.gateway.id
  }
}

# Local Network Gateway (represents AWS VPN endpoint)
variable "aws_vpn_tunnel1_ip" {
  description = "Outside IP of AWS VPN tunnel 1"
  type        = string
}

resource "azurerm_local_network_gateway" "aws" {
  name                = "aws-local-gw"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  gateway_address     = var.aws_vpn_tunnel1_ip
  address_space       = ["10.1.0.0/16"]
}

# VPN Connection
resource "azurerm_virtual_network_gateway_connection" "to_aws" {
  name                       = "azure-to-aws-vpn"
  location                   = azurerm_resource_group.main.location
  resource_group_name        = azurerm_resource_group.main.name
  type                       = "IPsec"
  virtual_network_gateway_id = azurerm_virtual_network_gateway.vpn.id
  local_network_gateway_id   = azurerm_local_network_gateway.aws.id
  shared_key                 = "YourPreSharedKey123!"  # Must match AWS
}

# Route table for cross-cloud traffic
resource "azurerm_route_table" "main" {
  name                = "cross-cloud-rt"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
}

resource "azurerm_route" "to_aws" {
  name                   = "to-aws"
  resource_group_name    = azurerm_resource_group.main.name
  route_table_name       = azurerm_route_table.main.name
  address_prefix         = "10.1.0.0/16"
  next_hop_type          = "VirtualNetworkGateway"
}

resource "azurerm_subnet_route_table_association" "subnet1" {
  subnet_id      = azurerm_subnet.subnet2.id
  route_table_id = azurerm_route_table.main.id
}

# NSG for cross-cloud traffic
resource "azurerm_network_security_group" "cross_cloud" {
  name                = "cross-cloud-nsg"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
}

resource "azurerm_network_security_rule" "allow_aws_icmp" {
  name                        = "allow-aws-icmp"
  priority                    = 100
  direction                   = "Inbound"
  access                      = "Allow"
  protocol                    = "Icmp"
  source_port_range           = "*"
  destination_port_range      = "*"
  source_address_prefix       = "10.1.0.0/16"
  destination_address_prefix  = "*"
  resource_group_name         = azurerm_resource_group.main.name
  network_security_group_name = azurerm_network_security_group.cross_cloud.name
}

resource "azurerm_network_security_rule" "allow_aws_ssh" {
  name                        = "allow-aws-ssh"
  priority                    = 110
  direction                   = "Inbound"
  access                      = "Allow"
  protocol                    = "Tcp"
  source_port_range           = "*"
  destination_port_range      = "22"
  source_address_prefix       = "10.1.0.0/16"
  destination_address_prefix  = "*"
  resource_group_name         = azurerm_resource_group.main.name
  network_security_group_name = azurerm_network_security_group.cross_cloud.name
}

# Test VM in subnet 2
resource "azurerm_network_interface" "test" {
  name                = "test-nic"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  ip_configuration {
    name                          = "internal"
    subnet_id                     = azurerm_subnet.subnet2.id
    private_ip_address_allocation = "Dynamic"
  }
}

resource "azurerm_linux_virtual_machine" "test" {
  name                = "azure-test-vm"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  size                = "Standard_B1s"
  admin_username      = "adminuser"
  network_interface_ids = [azurerm_network_interface.test.id]
  admin_ssh_key {
    username   = "adminuser"
    public_key = file("~/.ssh/id_rsa.pub")
  }
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

output "azure_vm_private_ip" {
  value = azurerm_linux_virtual_machine.test.private_ip_address
}

output "azure_vpn_gateway_ip" {
  value = azurerm_public_ip.vpn.ip_address
}
```

## Verification Steps

```bash
# 1. Apply AWS side first (use placeholder for Azure IP)
terraform apply -target=aws_vpc.main -target=aws_vpn_gateway.main ...

# 2. Apply Azure side (use AWS tunnel IP from step 1 output)
terraform apply

# 3. Update AWS Customer Gateway with actual Azure VPN IP
# (Re-apply if needed)

# 4. Check AWS VPN status
aws ec2 describe-vpn-connections \
  --filters Name=tag:Name,Values=aws-to-azure-vpn \
  --query 'VpnConnections[0].VgwTelemetry'

# 5. Check Azure VPN status
az network vpn-connection show \
  --name azure-to-aws-vpn \
  --resource-group cross-cloud-rg \
  --query 'connectionStatus'

# 6. Test connectivity
# From AWS instance:
ping <azure_vm_private_ip>
# From Azure instance:
ping <aws_instance_private_ip>

# 7. Measure latency
ping -c 100 <remote_private_ip>
```

## Trade-Offs: VPN vs Dedicated Interconnect

| Factor | Site-to-Site VPN | Dedicated Interconnect |
|--------|-----------------|----------------------|
| Setup time | Minutes | Weeks to months |
| Cost | Low (data transfer only) | High ($0.02-0.10/GB + port fees) |
| Latency | Internet-dependent (20-80ms) | Consistent (5-15ms) |
| Bandwidth | Limited (~1.25 Gbps per tunnel) | 10-100 Gbps |
| Reliability | Depends on internet path | SLA-backed |
| Encryption | Built-in (IPsec) | Must add application-level |

For production multi-cloud, start with VPN and upgrade to dedicated
interconnect when bandwidth or latency requirements justify the cost.
