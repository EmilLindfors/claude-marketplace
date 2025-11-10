---
description: Check code for opportunities to use modern Rust patterns
---

You are analyzing Rust code to identify opportunities for using modern patterns from Rust 2024 Edition.

## Your Task

Scan the codebase for patterns that could be modernized and provide a detailed report with recommendations.

## Steps

1. **Check Edition and Version**

   First, verify the project setup:

   Read Cargo.toml:
   ```
   Current Configuration:
   - Edition: [edition]
   - Rust version: [rust-version if set]
   - Toolchain: [rustc --version]
   ```

   If not on 2024:
   ```
   ℹ️ Project is using edition [current]. Consider upgrading to 2024 Edition
   to use latest features. Use `/rust-upgrade-edition` to upgrade.
   ```

2. **Scan for Nested If-Let Patterns**

   Search for nested if-let that could use let chains:

   Pattern to find:
   ```rust
   if let Pattern1 = expr1 {
       if let Pattern2 = expr2 {
           // Could be flattened with let chains
       }
   }
   ```

   Use Grep to search for:
   - `if let` followed by another `if let` in same scope
   - Multiple levels of nesting

   Report:
   ```
   ## Let Chain Opportunities (5):

   ### High Priority (deep nesting):
   - src/user.rs:42 - 4 levels of nested if-let
     Current: if let -> if let -> if let -> if let
     Suggestion: Use let chains to flatten

   - src/api.rs:88 - 3 levels of nested if-let
     Current: Multiple Option unwrapping
     Suggestion: Combine with && in single if

   ### Medium Priority:
   - src/validate.rs:15 - 2 levels with boolean check
   - src/handler.rs:102 - if let with additional condition
   ```

3. **Scan for Manual Async Closures**

   Look for patterns like:
   ```rust
   .map(|x| {
       let x = x.clone();
       async move { ... }
   })
   ```

   Report:
   ```
   ## Async Closure Opportunities (3):

   - src/tasks.rs:28
     Current: Manual async move wrapper
     Suggestion: Use async |x| { ... } syntax

   - src/jobs.rs:45
     Current: Clone before async move
     Suggestion: Async closure can borrow directly
   ```

4. **Scan for Const Function Opportunities**

   Look for functions that:
   - Contain only const-safe operations
   - Are used in const contexts
   - Could be evaluated at compile time

   Patterns to check:
   - Pure computation functions
   - Functions with no I/O or allocations
   - Hash functions, calculations

   Report:
   ```
   ## Const Function Opportunities (4):

   - src/config.rs:10 - calculate_buffer_size
     Current: fn calculate_buffer_size(n: usize) -> usize
     Suggestion: Add const keyword for compile-time eval
     Impact: Can be used in const/static initialization

   - src/hash.rs:25 - hash_string
     Current: Regular function
     Suggestion: Make const fn for compile-time hashing
   ```

5. **Scan for Manual Iterator Implementations**

   Look for:
   - struct implementing Iterator trait
   - Complex next() implementations
   - State machine patterns

   Report:
   ```
   ## Gen Block Opportunities (2):

   - src/iter.rs:15 - FibonacciIterator
     Current: 25 lines of Iterator impl
     Suggestion: Replace with 8-line gen block
     Benefit: Simpler, more maintainable

   - src/tree.rs:48 - TreeTraversal
     Current: Complex state machine
     Suggestion: Recursive gen block
   ```

6. **Scan for Match Pattern Improvements**

   Look for:
   - Match on references with moves
   - Unclear binding modes
   - Patterns that would benefit from 2024 ergonomics

   Report:
   ```
   ## Match Ergonomics Opportunities (3):

   - src/process.rs:65
     Current: Match with clone to avoid move
     Suggestion: Use 2024 ergonomics for in-place modify

   - src/validate.rs:42
     Current: Explicit ref patterns
     Suggestion: Can be simplified with 2024 ergonomics
   ```

7. **Scan for While-Let Chains**

   Look for while-let with additional conditions:

   Pattern:
   ```rust
   while let Some(item) = iterator.next() {
       if condition {
           // Could use while let with &&
       }
   }
   ```

   Report:
   ```
   ## While-Let Chain Opportunities (2):

   - src/parser.rs:35
     Current: while let with nested if
     Suggestion: Combine with && condition
   ```

8. **Scan for Never Type Opportunities**

   Look for functions that never return:
   - Functions ending with process::exit
   - Functions that always panic
   - Infinite loops without return

   Report:
   ```
   ## Never Type (!) Opportunities (3):

   - src/error.rs:25 - fatal_error
     Current: fn fatal_error(msg: &str)
     Suggestion: Change to fn fatal_error(msg: &str) -> !
     Benefit: Compiler knows function doesn't return
   ```

9. **Check for Outdated Idioms**

   Look for patterns that are outdated:
   - try! macro instead of ?
   - match instead of if let
   - Unnecessary type annotations
   - Old-style error handling

   Report:
   ```
   ## Outdated Idioms (2):

   - src/legacy.rs:15
     Current: try!(expression)
     Suggestion: Use ? operator

   - src/utils.rs:88
     Current: match on Result with Ok/Err
     Suggestion: Use if let Ok(...) for simpler case
   ```

10. **Generate Comprehensive Report**

    ```
    ✅ Pattern Check Complete

    ## Summary:

    📊 Total Opportunities: 19

    ### By Category:
    - Let Chains: 5 opportunities (15 nested levels total)
    - Async Closures: 3 opportunities
    - Const Functions: 4 opportunities
    - Gen Blocks: 2 opportunities
    - Match Ergonomics: 3 opportunities
    - While-Let Chains: 2 opportunities

    ### Priority:
    - High: 7 (deep nesting, clarity improvements)
    - Medium: 8 (performance, modern idioms)
    - Low: 4 (minor improvements)

    ## Detailed Findings:

    ### 1. Let Chains (High Priority)

    **src/user.rs:42** (4 levels nested)
    ```rust
    // Current
    if let Some(user) = get_user(id) {
        if let Some(profile) = user.profile {
            if profile.is_active {
                if let Some(email) = profile.email {
                    send_email(&email);
                }
            }
        }
    }

    // Suggested
    if let Some(user) = get_user(id)
        && let Some(profile) = user.profile
        && profile.is_active
        && let Some(email) = profile.email
    {
        send_email(&email);
    }
    ```

    **Impact:** Much clearer, reduces nesting from 4 to 1 level

    ### 2. Const Functions (Medium Priority)

    **src/config.rs:10**
    ```rust
    // Current
    fn calculate_buffer_size(items: usize) -> usize {
        items * std::mem::size_of::<Item>()
    }

    // Suggested
    const fn calculate_buffer_size(items: usize) -> usize {
        items * std::mem::size_of::<Item>()
    }

    // Enables
    const BUFFER_SIZE: usize = calculate_buffer_size(1000);
    static BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    ```

    **Impact:** Compile-time computation, better performance

    ### 3. Gen Blocks (Medium Priority)

    **src/iter.rs:15**
    ```rust
    // Current: 25 lines
    struct FibIterator { a: u64, b: u64 }
    impl Iterator for FibIterator {
        type Item = u64;
        fn next(&mut self) -> Option<u64> {
            let current = self.a;
            self.a = self.b;
            self.b = current + self.b;
            Some(current)
        }
    }

    // Suggested: 8 lines
    fn fibonacci() -> impl Iterator<Item = u64> {
        gen {
            let (mut a, mut b) = (0, 1);
            loop {
                yield a;
                (a, b) = (b, a + b);
            }
        }
    }
    ```

    **Impact:** 60% less code, more maintainable

    ## Edition Status:

    Current: edition = "2021"
    ⚠️ To use these features, upgrade to edition = "2024"

    Use: `/rust-upgrade-edition`

    ## Recommendations:

    1. **Immediate:** Upgrade to Rust 2024 Edition
    2. **High Priority:** Apply let chain modernizations (5 locations)
    3. **Medium Priority:** Convert to const functions (4 locations)
    4. **Medium Priority:** Simplify with gen blocks (2 locations)
    5. **Consider:** Async closure updates when appropriate

    ## Estimated Impact:

    - **Readability:** +40% (reduced nesting)
    - **Code reduction:** -15% (simpler patterns)
    - **Performance:** +5% (compile-time computation)
    - **Maintainability:** +30% (modern idioms)

    ## Next Steps:

    ```bash
    # 1. Upgrade edition
    /rust-upgrade-edition

    # 2. Apply modernizations
    /rust-modernize

    # 3. Run tests
    cargo test
    ```

    ## Need Help?

    - Ask `rust-modern-expert` for detailed guidance
    - Use `/rust-modernize` to apply changes automatically
    - See examples in CONTEXT.md
    ```

## Search Patterns

Use Grep to find these patterns:

### Nested If-Let
```
Pattern: if let.*\{[^}]*if let
```

### Manual Async Closures
```
Pattern: async move
```

### Non-const Functions
```
Pattern: fn .+\(.*\) -> (usize|i32|u32|bool)
Filter out those already const
```

### Manual Iterators
```
Pattern: impl Iterator for
```

## Scoring System

Assign priority based on:

**High Priority:**
- 3+ levels of nesting
- Functions in hot paths
- Widely used patterns

**Medium Priority:**
- 2 levels of nesting
- Const-eligible functions
- Iterator simplifications

**Low Priority:**
- Style improvements
- Minor simplifications

## Important Notes

- Only suggest changes compatible with current edition
- Note if edition upgrade is required
- Estimate impact (readability, performance, maintainability)
- Prioritize changes by value
- Provide before/after examples

## After Completion

Ask the user:
1. Do you want to apply these modernizations?
2. Should we start with high priority items?
3. Do you want to upgrade the edition first?
4. Would you like detailed explanations for any items?
