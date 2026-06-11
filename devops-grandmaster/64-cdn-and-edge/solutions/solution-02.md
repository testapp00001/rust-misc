# Solution 02: Configure CloudFront for a Web Application

## Part A -- S3 Bucket and Origin Access

### S3 Bucket Creation and File Upload

```bash
# Create the bucket
aws s3api create-bucket \
  --bucket my-app-assets-unique123 \
  --region us-east-1

# Upload sample files
aws s3 sync ./build/ s3://my-app-assets-unique123/ \
  --cache-control "public, max-age=86400"

# Set longer cache headers for static assets
aws s3 cp s3://my-app-assets-unique123/assets/ \
  s3://my-app-assets-unique123/assets/ \
  --recursive \
  --metadata-directive REPLACE \
  --cache-control "public, max-age=604800, immutable"
```

### Origin Access Control (OAC)

OAC replaces the legacy Origin Access Identity. It uses AWS Signature Version 4 to
authenticate CloudFront requests to S3, supporting all S3 features including SSE-KMS.

S3 bucket policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowCloudFrontServicePrincipal",
      "Effect": "Allow",
      "Principal": {
        "Service": "cloudfront.amazonaws.com"
      },
      "Action": "s3:GetObject",
      "Resource": "arn:aws:s3:::my-app-assets-unique123/*",
      "Condition": {
        "StringEquals": {
          "AWS:SourceArn": "arn:aws:cloudfront::ACCOUNT_ID:distribution/DISTRIBUTION_ID"
        }
      }
    }
  ]
}
```

The `Condition` block with `AWS:SourceArn` ensures that only your specific CloudFront
distribution can access the bucket, not any other CloudFront distribution.

## Part B -- CloudFront Distribution

The distribution configuration in Terraform:

```hcl
resource "aws_cloudfront_distribution" "app" {
  enabled             = true
  is_ipv6_enabled     = true
  default_root_object = "index.html"
  http_version        = "http2and3"
  price_class         = "PriceClass_100"

  aliases = ["app.example.com"]

  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate.cert.arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }

  # S3 origin with OAC
  origin {
    domain_name              = aws_s3_bucket.app.bucket_regional_domain_name
    origin_id                = "s3-app"
    origin_access_control_id = aws_cloudfront_origin_access_control.app.id
  }

  # Default behavior: cache HTML for 1 day
  default_cache_behavior {
    allowed_methods        = ["GET", "HEAD", "OPTIONS"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "s3-app"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    cache_policy_id = aws_cloudfront_cache_policy.default.id

    response_headers_policy_id = aws_cloudfront_response_headers_policy.security.id
  }

  # Static assets: cache for 7 days with immutable
  ordered_cache_behavior {
    path_pattern           = "/assets/*"
    allowed_methods        = ["GET", "HEAD"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "s3-app"
    viewer_protocol_policy = "redirect-to-https"
    compress               = true

    cache_policy_id = aws_cloudfront_cache_policy.assets.id
  }

  # SPA error handling
  dynamic "custom_error_response" {
    for_each = [403, 404]
    content {
      error_code            = custom_error_response.value
      response_page_path    = "/index.html"
      response_code         = 200
      error_caching_min_ttl = 300
    }
  }
}
```

## Part C -- Custom Error Pages

### Why This Is Necessary for SPAs

In a traditional multi-page application, each URL maps to a server-side route that
returns a unique HTML page. In a single-page application, the browser loads
`index.html` once, and JavaScript handles all routing client-side.

When a user navigates directly to `/about` (or refreshes the page), the browser sends
`GET /about` to CloudFront. CloudFront forwards this to S3, which has no object named
`about` -- it returns a 403 Forbidden. Without custom error handling, the user sees an
error page instead of the SPA.

The custom error response configuration intercepts 403 and 404 responses, serves
`/index.html` with a 200 status code, and the SPA's client-side router (React Router,
Vue Router) renders the correct page based on the URL in the browser's address bar.

The TTL of 300 seconds means CloudFront caches the error response for 5 minutes. This
is intentional: `index.html` rarely changes, and caching the error response reduces
origin load for common 403/404 scenarios.

## Part D -- Cache Policy and Response Headers

### Cache Policy

```hcl
resource "aws_cloudfront_cache_policy" "default" {
  name        = "app-default-cache"
  comment     = "Default cache policy for SPA"
  default_ttl = 86400
  max_ttl     = 31536000
  min_ttl     = 0

  parameters_in_cache_key_and_forwarded_to_origin {
    cookies_config {
      cookie_behavior = "none"
    }
    headers_config {
      header_behavior = "whitelist"
      headers {
        items = ["Accept", "Accept-Encoding"]
      }
    }
    query_strings_config {
      query_string_behavior = "none"
    }
    enable_accept_encoding_brotli = true
    enable_accept_encoding_gzip   = true
  }
}
```

### Response Headers Policy

```hcl
resource "aws_cloudfront_response_headers_policy" "security" {
  name    = "security-headers"
  comment = "Security headers for all responses"

  security_headers_config {
    strict_transport_security {
      access_control_max_age_sec = 31536000
      include_subdomains         = true
      preload                    = true
      override                   = true
    }
    content_type_options {
      override = true
    }
    frame_options {
      frame_option = "DENY"
      override     = true
    }
    xss_protection {
      mode_block = true
      protection = true
      override   = true
    }
  }

  custom_headers_config {
    items {
      header   = "Content-Security-Policy"
      value    = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'"
      override = true
    }
    items {
      header   = "Referrer-Policy"
      value    = "strict-origin-when-cross-origin"
      override = true
    }
  }
}
```

### Why Accept-Encoding in the Cache Key

Brotli-compressed responses are 15-25% smaller than Gzip. If `Accept-Encoding` is not
in the cache key, a Brotli-capable client's request might be served a Gzip-cached
response (or vice versa). By including it in the cache key, CloudFront stores separate
variants and serves the optimal compression for each client.

## Part E -- Terraform Representation

The complete Terraform configuration:

```hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}

# --- S3 ---

resource "aws_s3_bucket" "app" {
  bucket = "my-app-assets-unique123"
}

resource "aws_s3_bucket_public_access_block" "app" {
  bucket                  = aws_s3_bucket.app.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_policy" "app" {
  bucket = aws_s3_bucket.app.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Sid       = "AllowCloudFrontServicePrincipal"
      Effect    = "Allow"
      Principal = { Service = "cloudfront.amazonaws.com" }
      Action    = "s3:GetObject"
      Resource  = "${aws_s3_bucket.app.arn}/*"
      Condition = {
        StringEquals = {
          "AWS:SourceArn" = aws_cloudfront_distribution.app.arn
        }
      }
    }]
  })
}

# --- OAC ---

resource "aws_cloudfront_origin_access_control" "app" {
  name                              = "app-oac"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# --- Cache Policies (defined above in Part D) ---
# --- Response Headers Policy (defined above in Part D) ---

# --- CloudFront Distribution (defined above in Part B) ---

output "distribution_domain" {
  value = aws_cloudfront_distribution.app.domain_name
}
```

Run `terraform init && terraform plan` to verify the configuration. The plan should
show 6-8 resources to create depending on whether you also create the ACM certificate
and Route 53 records.

## Common Mistakes to Avoid

1. **Using OAI instead of OAC**: OAI is deprecated and does not support SSE-KMS
   encrypted objects. Always use OAC for new distributions.

2. **Forgetting the bucket policy**: Creating the OAC alone does not grant access.
   The S3 bucket policy must explicitly allow the CloudFront service principal.

3. **Not setting `default_root_object`**: Without this, requests to the bare domain
   (`https://app.example.com/`) return the S3 listing or a 403 instead of
   `index.html`.

4. **Caching API responses when you should not**: The `/api/*` behavior must forward
   all headers, methods, and query strings. If you accidentally cache API responses,
   users may see stale or other users' data.

5. **Missing `override = true` in response headers**: Without `override`, the origin
   can set weaker security headers that take precedence over your policy.

## Key Takeaway

Configuring CloudFront for a modern web application involves more than pointing a
distribution at an S3 bucket. The OAC secures origin access, custom error pages enable
SPA routing, cache policies optimize hit ratios, and response headers enforce a
security baseline. Each configuration choice has a direct impact on security,
performance, and user experience.
