# ============================================================================
# DSA Learning Project — Makefile
# ============================================================================
# Quick reference:
#   make list                          — List all topics
#   make test TOPIC=01-arrays-and-hashing                — Test a topic (your solution)
#   make test TOPIC=01-arrays-and-hashing PROB=p01_two_sum  — Test one problem
#   make solution TOPIC=01-arrays-and-hashing            — Test reference solution
#   make bench TOPIC=01-arrays-and-hashing               — Run benchmarks
#   make bench TOPIC=01-arrays-and-hashing PROB=p01_two_sum — Bench one problem
#   make test-all                      — Run all tests (your solutions)
#   make solution-all                  — Run all tests (reference solutions)
# ============================================================================

# ── List all topics ─────────────────────────────────────────────────────────
.PHONY: list
list:
	@echo "Available topics:"
	@echo ""
	@ls -1 | grep '^[0-9]' | while read d; do \
		num=$$(echo "$$d" | cut -d'-' -f1); \
		name=$$(echo "$$d" | sed 's/^[0-9]*-//'); \
		count=$$(find $$d/src -maxdepth 1 -name 'p*.rs' 2>/dev/null | wc -l); \
		printf "  %-3s %-35s (%d problems)\n" "$$num" "$$name" "$$count"; \
	done
	@echo ""
	@echo "Usage: make test TOPIC=01-arrays-and-hashing"
	@echo "       make test TOPIC=01-arrays-and-hashing PROB=p01_two_sum"

# ── Test your solution ──────────────────────────────────────────────────────
# Tests fail with todo!() until you implement your solution.
.PHONY: test
test:
ifndef TOPIC
	@echo "Usage: make test TOPIC=01-arrays-and-hashing [PROB=p01_two_sum]"
	@exit 1
endif
ifdef PROB
	cd $(TOPIC) && cargo test $(PROB) -- --nocapture
else
	cd $(TOPIC) && cargo test -- --nocapture
endif

# ── Test reference solution ─────────────────────────────────────────────────
# Compiles and tests the reference implementation behind --features solution.
.PHONY: solution
solution:
ifndef TOPIC
	@echo "Usage: make solution TOPIC=01-arrays-and-hashing [PROB=p01_two_sum]"
	@exit 1
endif
ifdef PROB
	cd $(TOPIC) && cargo test --features solution $(PROB) -- --nocapture
else
	cd $(TOPIC) && cargo test --features solution -- --nocapture
endif

# ── Run benchmarks ──────────────────────────────────────────────────────────
# Runs ignored benchmark tests with timing output.
.PHONY: bench
bench:
ifndef TOPIC
	@echo "Usage: make bench TOPIC=01-arrays-and-hashing [PROB=p01_two_sum]"
	@exit 1
endif
ifdef PROB
	cd $(TOPIC) && cargo test --features solution $(PROB) -- --ignored --nocapture
else
	cd $(TOPIC) && cargo test --features solution -- --ignored --nocapture
endif

# ── Run ALL tests across all topics ─────────────────────────────────────────
.PHONY: test-all
test-all:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Running ALL tests (your solutions)"
	@echo "═══════════════════════════════════════════════════════════════"
	cargo test --workspace

# ── Run ALL reference solution tests ────────────────────────────────────────
.PHONY: solution-all
solution-all:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Running ALL tests (reference solutions)"
	@echo "═══════════════════════════════════════════════════════════════"
	@for dir in [0-9]*/; do \
		topic=$$(basename "$$dir"); \
		echo ""; \
		echo "▸ $$topic"; \
		echo "───────────────────────────────────────────────"; \
		cd "$$dir" && cargo test --features solution -- --nocapture 2>&1; \
		cd ..; \
	done

# ── Run ALL benchmarks ──────────────────────────────────────────────────────
.PHONY: bench-all
bench-all:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Running ALL benchmarks"
	@echo "═══════════════════════════════════════════════════════════════"
	@for dir in [0-9]*/; do \
		topic=$$(basename "$$dir"); \
		echo ""; \
		echo "▸ $$topic"; \
		echo "───────────────────────────────────────────────"; \
		cd "$$dir" && cargo test --features solution -- --ignored --nocapture 2>&1; \
		cd ..; \
	done

# ── List problems in a topic ────────────────────────────────────────────────
.PHONY: problems
problems:
ifndef TOPIC
	@echo "Usage: make problems TOPIC=01-arrays-and-hashing"
	@exit 1
endif
	@echo "Problems in $(TOPIC):"
	@echo ""
	@ls -1 $(TOPIC)/src/p*.rs | xargs -I{} basename {} .rs | sort

# ── Show topic README ───────────────────────────────────────────────────────
.PHONY: readme
readme:
ifndef TOPIC
	@echo "Usage: make readme TOPIC=01-arrays-and-hashing"
	@exit 1
endif
	@cat $(TOPIC)/README.md

# ── Help ────────────────────────────────────────────────────────────────────
.PHONY: help
help:
	@echo "DSA Learning Project"
	@echo ""
	@echo "Commands:"
	@echo "  make list                                List all topics"
	@echo "  make test   TOPIC=<topic> [PROB=<prob>]   Test your solution"
	@echo "  make solution TOPIC=<topic> [PROB=<prob>] Test reference solution"
	@echo "  make bench  TOPIC=<topic> [PROB=<prob>]   Run benchmarks"
	@echo "  make problems TOPIC=<topic>               List problems in topic"
	@echo "  make readme  TOPIC=<topic>                Show topic README"
	@echo "  make test-all                             Test all (your solutions)"
	@echo "  make solution-all                         Test all (reference)"
	@echo "  make bench-all                            Benchmark all"
	@echo ""
	@echo "Examples:"
	@echo "  make test TOPIC=01-arrays-and-hashing"
	@echo "  make test TOPIC=01-arrays-and-hashing PROB=p01_two_sum"
	@echo "  make bench TOPIC=07-trees PROB=p01_invert_tree"
