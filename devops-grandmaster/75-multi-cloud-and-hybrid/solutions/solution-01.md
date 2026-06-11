# Solution 01: Multi-Cloud vs Hybrid Cloud Trade-Offs

## Part A: Scenario Analysis

### Scenario 1 -- FinTech Startup: Single-Cloud (AWS)

A 20-person startup should adopt a single-cloud strategy on AWS. The team
already has strong AWS expertise, which eliminates the learning curve of a
second provider. AWS has robust PCI-DSS compliance tooling (AWS Artifact,
Config Rules, Security Hub) that meets their regulatory needs. With only 20
people, the operational overhead of managing infrastructure on two clouds
would consume engineering time better spent on the product. AWS's global
regions cover both US and EU latency requirements. The risk of vendor lock-in
is real but is a problem to solve later when the team and revenue justify it.
Adding GCP or Azure now would double their infrastructure management burden
without proportional benefit.

### Scenario 2 -- Global Media Company: Multi-Cloud

A global media company with 40-country operations and existing on-premises
infrastructure needs a strategic multi-cloud approach. The on-premises
rendering farms should stay on-premises and connect to the cloud(s) via
hybrid networking -- rendering workloads are latency-sensitive and represent
sunk capital. For AI/ML, they should use the best service for each use case:
AWS SageMaker for some workloads, GCP Vertex AI for others (especially if
they use TensorFlow or need TPUs), and Azure Cognitive Services for
language/speech tasks. Data residency laws in 40 countries mean they need
cloud presence in many regions, which no single provider covers optimally.
The complexity is justified by their scale -- they likely have a dedicated
platform team to manage it.

### Scenario 3 -- Government Agency: Hybrid

A federal agency with classified and unclassified workloads needs a hybrid
strategy. Classified workloads remain on air-gapped on-premises
infrastructure -- this is non-negotiable. Unclassified workloads move to a
FedRAMP-authorized cloud (AWS GovCloud, Azure Government, or GCP). The key
requirement is unified identity and policy management across both
environments. Use a federated identity solution (e.g., Active Directory
federated to both on-prem and cloud IAM) and a policy engine like Open
Policy Agent (OPA) that enforces consistent rules regardless of where
workloads run. A single cloud provider is sufficient for unclassified
workloads -- multi-cloud adds complexity without clear benefit in a
government context where procurement and compliance are already complex.

### Scenario 4 -- E-Commerce Platform: Careful Multi-Cloud (Strategic)

This is the trickiest scenario. The heavy use of AWS-native services (Lambda,
DynamoDB, SQS) means true multi-cloud would require significant refactoring.
The right approach is NOT to blindly replicate everything on Azure. Instead:

1. **Identify the blast radius**: If AWS us-east-1 goes down, which services
   are affected? Lambda and DynamoDB are regional. SQS is regional.
2. **Build a read replica on Azure**: Use DynamoDB Global Tables to replicate
   to another AWS region first (cheaper and simpler than cross-cloud). For
   true multi-cloud, set up a PostgreSQL database on Azure as a fallback
   data store.
3. **Use multi-cloud DNS**: Route 53 health checks that failover to an Azure
   static site or CDN-served version.
4. **Negotiation leverage**: The threat of multi-cloud is more valuable than
   the reality. Use the Azure exploration as a negotiation tool, but do not
   rush to implement.

The recommendation is **strategic multi-cloud** for negotiation leverage and
a read-replica DR strategy, but NOT a full port of Lambda/DynamoDB to Azure.

### Scenario 5 -- Healthcare Provider: Hybrid

The hospital network should use a hybrid strategy. Patient records stay
on-premises due to the HIPAA compliance interpretation (even though major
clouds are HIPAA-compliant, the organizational policy mandates on-prem).
Analytics and ML workloads move to the cloud -- they are bursty and benefit
from cloud elasticity. Use AWS or GCP for the analytics platform (both have
strong healthcare analytics offerings). Connect the two environments via
VPN or dedicated interconnect. Unified security is achieved through:

- A single identity provider (e.g., Okta or Azure AD) federated to both
  environments
- Consistent encryption policies (TLS everywhere, encryption at rest in
  both locations)
- A SIEM that aggregates logs from both environments
- Network segmentation that ensures patient data never traverses the public
  internet

## Part B: Trade-Off Matrix

| Dimension | Single-Cloud | Multi-Cloud | Hybrid |
|-----------|-------------|-------------|--------|
| Operational complexity | Low | High | Medium-High |
| Vendor lock-in risk | High | Low | Medium |
| Talent/skill requirements | Low | High | High |
| Network complexity | Low | High | Medium |
| Cost optimization potential | Medium | High | Medium |
| Disaster recovery capability | Medium | High | Medium-High |
| Compliance flexibility | Low | High | High |
| Time to market | High (fast) | Low (slow) | Medium |

Key observations:
- Single-cloud wins on time to market and talent requirements.
- Multi-cloud wins on lock-in risk, cost optimization, and DR capability.
- Hybrid wins on compliance flexibility but has moderate network complexity.
- Multi-cloud has the highest operational complexity -- this is the primary
  deterrent and must be justified by business requirements.

## Part C: Anti-Patterns Essay -- The Abstraction Tax

Building cloud-agnostic abstractions carries a hidden cost that is often
underestimated. The "abstraction tax" manifests in several ways:

**Performance penalties**: Cloud-agnostic storage abstractions (like storing
objects through a generic S3-compatible API) may not support all native
features. You lose S3 Select, Glacier lifecycle policies, or GCP's
autoclass. Your abstraction must either ignore these features (leaving
performance on the table) or implement a lowest-common-denominator subset.

**Operational complexity**: Every abstraction layer is a new thing to debug.
When your Terraform module fails to create a database, is the bug in your
abstraction layer or in the provider-specific implementation? Debugging
through abstraction layers requires deep knowledge of ALL providers, which
is rarer and more expensive than deep knowledge of one.

**Feature lag**: When AWS launches a new feature, your abstraction does not
support it until you update the abstraction layer, implement it for all
three providers, and test it. You are always behind the native experience.

**When is the tax worth it?** The abstraction tax is justified when:
1. You have a regulatory or business requirement for multi-cloud portability.
2. Your organization has the engineering capacity to maintain the
   abstraction layer.
3. The workload is standardized enough that the common interface covers
   80%+ of the use cases.
4. You are building a platform team that provides infrastructure as a
   service to internal teams.

The tax is NOT worth it when you have a small team, your workload is tightly
coupled to one cloud's unique features, or you are building the abstraction
"just in case" without a concrete multi-cloud requirement.

The best approach is to abstract at the right layer. Do not abstract low-level
infrastructure (use provider-native resources). Do abstract application-level
concepts (use Kubernetes, standard APIs, open formats). The sweet spot is
abstracting the things that are genuinely portable while embracing the things
that are not.
