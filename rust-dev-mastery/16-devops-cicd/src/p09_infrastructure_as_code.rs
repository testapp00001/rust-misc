//! # Infrastructure as Code for Rust Services
//!
//! Infrastructure as Code (IaC) ensures reproducible, version-controlled
//! infrastructure. This module covers patterns for managing Rust service
//! infrastructure using declarative configuration, idempotency, and
//! infrastructure validation.
//!
//! ## IaC Principles:
//!
//! 1. **Declarative**: Describe desired state, not steps
//! 2. **Idempotent**: Apply multiple times with same result
//! 3. **Versioned**: Infrastructure changes tracked in Git
//! 4. **Testable**: Validate before applying
//! 5. **Modular**: Reusable components

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a cloud provider resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_type: ResourceType,
    pub name: String,
    pub properties: HashMap<String, ResourceValue>,
    pub depends_on: Vec<String>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    ComputeInstance,
    Database,
    LoadBalancer,
    DnsRecord,
    StorageBucket,
    NetworkVpc,
    SecurityGroup,
    Certificate,
    Monitoring,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceValue {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<String>),
    Reference(String), // Reference to another resource's output
}

impl Resource {
    pub fn new(resource_type: ResourceType, name: &str) -> Self {
        Self {
            resource_type,
            name: name.into(),
            properties: HashMap::new(),
            depends_on: Vec::new(),
            tags: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: &str, value: ResourceValue) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    pub fn depends_on(mut self, resource_name: &str) -> Self {
        self.depends_on.push(resource_name.into());
        self
    }

    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }
}

/// Infrastructure stack representing a collection of resources.
#[derive(Debug, Serialize, Deserialize)]
pub struct InfrastructureStack {
    pub name: String,
    pub version: String,
    pub resources: Vec<Resource>,
    pub variables: HashMap<String, String>,
    pub outputs: HashMap<String, String>,
}

impl InfrastructureStack {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            resources: Vec::new(),
            variables: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.into(), value.into());
    }

    pub fn set_output(&mut self, key: &str, value: &str) {
        self.outputs.insert(key.into(), value.into());
    }

    /// Validate the infrastructure stack for common issues.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let resource_names: Vec<&str> = self.resources.iter().map(|r| r.name.as_str()).collect();

        // Check for duplicate resource names
        let mut seen = std::collections::HashSet::new();
        for name in &resource_names {
            if !seen.insert(name) {
                errors.push(ValidationError {
                    resource: name.to_string(),
                    message: format!("Duplicate resource name: {}", name),
                });
            }
        }

        // Check dependencies exist
        for resource in &self.resources {
            for dep in &resource.depends_on {
                if !resource_names.contains(&dep.as_str()) {
                    errors.push(ValidationError {
                        resource: resource.name.clone(),
                        message: format!("Dependency '{}' not found", dep),
                    });
                }
            }
        }

        // Check for circular dependencies
        if self.has_circular_dependencies() {
            errors.push(ValidationError {
                resource: self.name.clone(),
                message: "Circular dependency detected".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn has_circular_dependencies(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        for resource in &self.resources {
            if self.detect_cycle(&resource.name, &mut visited, &mut in_stack) {
                return true;
            }
        }
        false
    }

    fn detect_cycle(
        &self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
        in_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if in_stack.contains(name) {
            return true;
        }
        if visited.contains(name) {
            return false;
        }

        visited.insert(name.to_string());
        in_stack.insert(name.to_string());

        if let Some(resource) = self.resources.iter().find(|r| r.name == name) {
            for dep in &resource.depends_on {
                if self.detect_cycle(dep, visited, in_stack) {
                    return true;
                }
            }
        }

        in_stack.remove(name);
        false
    }

    /// Generate a dependency-ordered plan for resource creation.
    pub fn execution_plan(&self) -> Vec<&str> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for resource in &self.resources {
            self.visit_for_order(&resource.name, &mut visited, &mut order);
        }

        order
    }

    fn visit_for_order<'a>(
        &'a self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<&'a str>,
    ) {
        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        if let Some(resource) = self.resources.iter().find(|r| r.name == name) {
            for dep in &resource.depends_on {
                self.visit_for_order(dep, visited, order);
            }
            order.push(&resource.name);
        }
    }

    /// Generate Terraform HCL for the stack.
    pub fn to_terraform_hcl(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("# Infrastructure Stack: {}\n", self.name));
        output.push_str(&format!("# Version: {}\n\n", self.version));

        for (key, value) in &self.variables {
            output.push_str(&format!("variable \"{}\" {{\n  default = \"{}\"\n}}\n\n", key, value));
        }

        for resource in &self.resources {
            let tf_type = match resource.resource_type {
                ResourceType::ComputeInstance => "aws_instance",
                ResourceType::Database => "aws_db_instance",
                ResourceType::LoadBalancer => "aws_lb",
                ResourceType::DnsRecord => "aws_route53_record",
                ResourceType::StorageBucket => "aws_s3_bucket",
                ResourceType::NetworkVpc => "aws_vpc",
                ResourceType::SecurityGroup => "aws_security_group",
                ResourceType::Certificate => "aws_acm_certificate",
                ResourceType::Monitoring => "aws_cloudwatch_metric_alarm",
            };

            output.push_str(&format!("resource \"{}\" \"{}\" {{\n", tf_type, resource.name));
            for (key, value) in &resource.properties {
                output.push_str(&format!("  {} = {}\n", key, format_tf_value(value)));
            }
            if !resource.tags.is_empty() {
                output.push_str("  tags = {\n");
                for (key, value) in &resource.tags {
                    output.push_str(&format!("    {} = \"{}\"\n", key, value));
                }
                output.push_str("  }\n");
            }
            output.push_str("}\n\n");
        }

        for (key, value) in &self.outputs {
            output.push_str(&format!("output \"{}\" {{\n  value = {}\n}}\n\n", key, value));
        }

        output
    }
}

fn format_tf_value(value: &ResourceValue) -> String {
    match value {
        ResourceValue::String(s) => format!("\"{}\"", s),
        ResourceValue::Number(n) => n.to_string(),
        ResourceValue::Boolean(b) => b.to_string(),
        ResourceValue::List(items) => {
            let formatted: Vec<String> = items.iter().map(|i| format!("\"{}\"", i)).collect();
            format!("[{}]", formatted.join(", "))
        }
        ResourceValue::Reference(r) => r.clone(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub resource: String,
    pub message: String,
}

/// Pre-built infrastructure templates for common Rust service patterns.
pub struct InfrastructureTemplates;

impl InfrastructureTemplates {
    /// A standard web service stack with compute, database, and load balancer.
    pub fn web_service(
        name: &str,
        instance_type: &str,
        db_engine: &str,
    ) -> InfrastructureStack {
        let mut stack = InfrastructureStack::new(name, "1.0.0");

        stack.set_variable("environment", "production");
        stack.set_variable("region", "us-east-1");

        let vpc = Resource::new(ResourceType::NetworkVpc, "main_vpc")
            .with_property("cidr_block", ResourceValue::String("10.0.0.0/16".into()))
            .with_tag("Name", &format!("{}-vpc", name))
            .with_tag("Environment", "production");
        stack.add_resource(vpc);

        let sg = Resource::new(ResourceType::SecurityGroup, "app_sg")
            .with_property("name", ResourceValue::String(format!("{}-sg", name)))
            .with_property(
                "ingress_rules",
                ResourceValue::List(vec![
                    "80/tcp".into(),
                    "443/tcp".into(),
                    "8080/tcp".into(),
                ]),
            )
            .depends_on("main_vpc");
        stack.add_resource(sg);

        let instance = Resource::new(ResourceType::ComputeInstance, "app_server")
            .with_property(
                "instance_type",
                ResourceValue::String(instance_type.into()),
            )
            .with_property(
                "ami",
                ResourceValue::Reference("data.aws_ami.ubuntu.id".into()),
            )
            .depends_on("app_sg")
            .with_tag("Name", &format!("{}-server", name));
        stack.add_resource(instance);

        let db = Resource::new(ResourceType::Database, "app_database")
            .with_property("engine", ResourceValue::String(db_engine.into()))
            .with_property(
                "instance_class",
                ResourceValue::String("db.t3.medium".into()),
            )
            .with_property(
                "allocated_storage",
                ResourceValue::Number(100.0),
            )
            .depends_on("main_vpc")
            .with_tag("Name", &format!("{}-db", name));
        stack.add_resource(db);

        let lb = Resource::new(ResourceType::LoadBalancer, "app_lb")
            .with_property(
                "type",
                ResourceValue::String("application".into()),
            )
            .depends_on("app_sg")
            .with_tag("Name", &format!("{}-lb", name));
        stack.add_resource(lb);

        stack.set_output(
            "load_balancer_dns",
            "aws_lb.app_lb.dns_name",
        );
        stack.set_output(
            "database_endpoint",
            "aws_db_instance.app_database.endpoint",
        );

        stack
    }
}

/// Idempotency checker for infrastructure operations.
pub struct IdempotencyChecker;

impl IdempotencyChecker {
    /// Compare desired state with current state to determine needed changes.
    pub fn plan_changes(
        desired: &[Resource],
        current: &[Resource],
    ) -> InfrastructurePlan {
        let mut plan = InfrastructurePlan::new();

        let current_map: HashMap<&str, &Resource> =
            current.iter().map(|r| (r.name.as_str(), r)).collect();

        for desired_resource in desired {
            match current_map.get(desired_resource.name.as_str()) {
                None => {
                    plan.create.push(desired_resource.name.clone());
                }
                Some(current_resource) => {
                    if Self::resources_differ(desired_resource, current_resource) {
                        plan.update.push(desired_resource.name.clone());
                    }
                }
            }
        }

        let desired_names: std::collections::HashSet<&str> =
            desired.iter().map(|r| r.name.as_str()).collect();
        for current_resource in current {
            if !desired_names.contains(current_resource.name.as_str()) {
                plan.destroy.push(current_resource.name.clone());
            }
        }

        plan
    }

    fn resources_differ(a: &Resource, b: &Resource) -> bool {
        a.resource_type != b.resource_type || a.properties != b.properties
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfrastructurePlan {
    pub create: Vec<String>,
    pub update: Vec<String>,
    pub destroy: Vec<String>,
}

impl InfrastructurePlan {
    pub fn new() -> Self {
        Self {
            create: Vec::new(),
            update: Vec::new(),
            destroy: Vec::new(),
        }
    }

    pub fn is_noop(&self) -> bool {
        self.create.is_empty() && self.update.is_empty() && self.destroy.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "Plan: {} to create, {} to update, {} to destroy",
            self.create.len(),
            self.update.len(),
            self.destroy.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(ResourceType::ComputeInstance, "web-server")
            .with_property("instance_type", ResourceValue::String("t3.medium".into()))
            .with_tag("Environment", "production");

        assert_eq!(resource.name, "web-server");
        assert_eq!(resource.resource_type, ResourceType::ComputeInstance);
        assert!(resource.properties.contains_key("instance_type"));
        assert_eq!(resource.tags.get("Environment").unwrap(), "production");
    }

    #[test]
    fn test_resource_dependencies() {
        let resource = Resource::new(ResourceType::ComputeInstance, "app")
            .depends_on("vpc")
            .depends_on("security_group");

        assert_eq!(resource.depends_on.len(), 2);
        assert!(resource.depends_on.contains(&"vpc".to_string()));
    }

    #[test]
    fn test_infrastructure_stack_validation() {
        let mut stack = InfrastructureStack::new("test", "1.0.0");
        stack.add_resource(Resource::new(ResourceType::NetworkVpc, "vpc"));
        stack.add_resource(
            Resource::new(ResourceType::ComputeInstance, "app").depends_on("vpc"),
        );

        assert!(stack.validate().is_ok());
    }

    #[test]
    fn test_infrastructure_stack_duplicate_names() {
        let mut stack = InfrastructureStack::new("test", "1.0.0");
        stack.add_resource(Resource::new(ResourceType::NetworkVpc, "same-name"));
        stack.add_resource(Resource::new(ResourceType::ComputeInstance, "same-name"));

        let result = stack.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("Duplicate")));
    }

    #[test]
    fn test_infrastructure_stack_missing_dependency() {
        let mut stack = InfrastructureStack::new("test", "1.0.0");
        stack.add_resource(
            Resource::new(ResourceType::ComputeInstance, "app").depends_on("nonexistent"),
        );

        let result = stack.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("not found")));
    }

    #[test]
    fn test_infrastructure_stack_circular_dependency() {
        let mut stack = InfrastructureStack::new("test", "1.0.0");
        stack.add_resource(
            Resource::new(ResourceType::ComputeInstance, "a").depends_on("b"),
        );
        stack.add_resource(
            Resource::new(ResourceType::ComputeInstance, "b").depends_on("a"),
        );

        let result = stack.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_plan_ordering() {
        let mut stack = InfrastructureStack::new("test", "1.0.0");
        stack.add_resource(
            Resource::new(ResourceType::ComputeInstance, "app").depends_on("db"),
        );
        stack.add_resource(Resource::new(ResourceType::Database, "db").depends_on("vpc"));
        stack.add_resource(Resource::new(ResourceType::NetworkVpc, "vpc"));

        let plan = stack.execution_plan();
        let vpc_idx = plan.iter().position(|r| *r == "vpc").unwrap();
        let db_idx = plan.iter().position(|r| *r == "db").unwrap();
        let app_idx = plan.iter().position(|r| *r == "app").unwrap();

        assert!(vpc_idx < db_idx);
        assert!(db_idx < app_idx);
    }

    #[test]
    fn test_terraform_hcl_generation() {
        let mut stack = InfrastructureStack::new("test-stack", "1.0.0");
        stack.add_resource(
            Resource::new(ResourceType::ComputeInstance, "web")
                .with_property("instance_type", ResourceValue::String("t3.medium".into()))
                .with_tag("Name", "web-server"),
        );

        let hcl = stack.to_terraform_hcl();
        assert!(hcl.contains("aws_instance"));
        assert!(hcl.contains("web"));
        assert!(hcl.contains("t3.medium"));
        assert!(hcl.contains("web-server"));
    }

    #[test]
    fn test_terraform_hcl_with_variables() {
        let mut stack = InfrastructureStack::new("test", "1.0.0");
        stack.set_variable("region", "us-east-1");
        stack.add_resource(Resource::new(ResourceType::NetworkVpc, "vpc"));

        let hcl = stack.to_terraform_hcl();
        assert!(hcl.contains("variable \"region\""));
        assert!(hcl.contains("us-east-1"));
    }

    #[test]
    fn test_format_tf_value() {
        assert_eq!(format_tf_value(&ResourceValue::String("test".into())), "\"test\"");
        assert_eq!(format_tf_value(&ResourceValue::Number(42.0)), "42");
        assert_eq!(format_tf_value(&ResourceValue::Boolean(true)), "true");
        assert_eq!(
            format_tf_value(&ResourceValue::List(vec!["a".into(), "b".into()])),
            "[\"a\", \"b\"]"
        );
    }

    #[test]
    fn test_web_service_template() {
        let stack = InfrastructureTemplates::web_service("my-app", "t3.medium", "postgres");
        assert!(!stack.resources.is_empty());
        assert!(stack.validate().is_ok());

        let resource_types: Vec<&ResourceType> =
            stack.resources.iter().map(|r| &r.resource_type).collect();
        assert!(resource_types.contains(&&ResourceType::NetworkVpc));
        assert!(resource_types.contains(&&ResourceType::ComputeInstance));
        assert!(resource_types.contains(&&ResourceType::Database));
        assert!(resource_types.contains(&&ResourceType::LoadBalancer));
    }

    #[test]
    fn test_idempotency_no_changes() {
        let resources = vec![
            Resource::new(ResourceType::ComputeInstance, "app")
                .with_property("size", ResourceValue::String("t3.medium".into())),
        ];

        let plan = IdempotencyChecker::plan_changes(&resources, &resources);
        assert!(plan.is_noop());
    }

    #[test]
    fn test_idempotency_create_new() {
        let desired = vec![
            Resource::new(ResourceType::ComputeInstance, "app"),
            Resource::new(ResourceType::Database, "db"),
        ];
        let current = vec![
            Resource::new(ResourceType::ComputeInstance, "app"),
        ];

        let plan = IdempotencyChecker::plan_changes(&desired, &current);
        assert_eq!(plan.create.len(), 1);
        assert!(plan.create.contains(&"db".to_string()));
    }

    #[test]
    fn test_idempotency_destroy_removed() {
        let desired = vec![
            Resource::new(ResourceType::ComputeInstance, "app"),
        ];
        let current = vec![
            Resource::new(ResourceType::ComputeInstance, "app"),
            Resource::new(ResourceType::Database, "old-db"),
        ];

        let plan = IdempotencyChecker::plan_changes(&desired, &current);
        assert_eq!(plan.destroy.len(), 1);
        assert!(plan.destroy.contains(&"old-db".to_string()));
    }

    #[test]
    fn test_idempotency_update_changed() {
        let desired = vec![
            Resource::new(ResourceType::ComputeInstance, "app")
                .with_property("size", ResourceValue::String("t3.large".into())),
        ];
        let current = vec![
            Resource::new(ResourceType::ComputeInstance, "app")
                .with_property("size", ResourceValue::String("t3.medium".into())),
        ];

        let plan = IdempotencyChecker::plan_changes(&desired, &current);
        assert_eq!(plan.update.len(), 1);
    }

    #[test]
    fn test_plan_summary() {
        let plan = InfrastructurePlan {
            create: vec!["a".into()],
            update: vec!["b".into(), "c".into()],
            destroy: vec![],
        };
        assert_eq!(plan.summary(), "Plan: 1 to create, 2 to update, 0 to destroy");
    }
}
