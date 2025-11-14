---
description: Add a new prompt template to your MCP server
---

You are adding a new prompt to an existing MCP server project.

## Your Task

Guide the user through creating a well-designed prompt template that guides AI interactions.

## Steps

### 1. Gather Prompt Information

Ask the user:
```
I'll help you add a new prompt to your MCP server. Please provide:

1. Prompt name: (e.g., code_review, debug_session, api_design)
2. Description: What is this prompt for?
3. Arguments: What dynamic values does it need?
   - Argument name, description, required/optional
4. Context: What context should be included?
   - Project files, configuration, examples, guidelines
5. Style: What role/persona should the AI adopt?
```

### 2. Create Prompt Module

Generate `src/prompts/{prompt_name}.rs`:

```rust
use rmcp::prelude::*;
use serde::{{Deserialize, Serialize}};
use schemars::JsonSchema;
use std::collections::HashMap;
use crate::error::{{Error, Result}};

/// Argument definition for {prompt_name}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct {PromptName}Argument {{
    pub name: String,
    pub description: String,
    pub required: bool,
}}

/// {Prompt description}
pub struct {PromptName}Prompt {{
    // Context providers
    config: AppConfig,
}}

impl {PromptName}Prompt {{
    pub fn new(config: AppConfig) -> Self {{
        Self {{ config }}
    }}

    /// Get prompt metadata
    pub fn get_info(&self) -> PromptInfo {{
        PromptInfo {{
            name: "{prompt_name}".to_string(),
            description: Some("{Full prompt description}".to_string()),
            arguments: vec![
                {PromptName}Argument {{
                    name: "{arg1}".to_string(),
                    description: "{Arg1 description}".to_string(),
                    required: true,
                }},
                {PromptName}Argument {{
                    name: "{arg2}".to_string(),
                    description: "{Arg2 description}".to_string(),
                    required: false,
                }},
            ],
        }}
    }}

    /// Generate prompt with arguments
    pub async fn generate(
        &self,
        arguments: HashMap<String, String>,
    ) -> Result<PromptResponse> {{
        // Validate required arguments
        let {arg1} = arguments
            .get("{arg1}")
            .ok_or_else(|| Error::InvalidInput {{
                field: "{arg1}".to_string(),
                message: "Required argument missing".to_string(),
            }})?;

        let {arg2} = arguments
            .get("{arg2}")
            .map(|s| s.as_str())
            .unwrap_or("{default_value}");

        // Load context if needed
        let context = self.load_context(&arguments).await?;

        // Build messages
        let messages = vec![
            PromptMessage {{
                role: Role::System,
                content: self.build_system_message(&context),
            }},
            PromptMessage {{
                role: Role::User,
                content: self.build_user_message({arg1}, {arg2}, &context),
            }},
        ];

        Ok(PromptResponse {{
            description: Some(format!("{prompt_name} for {{}}", {arg1})),
            messages,
        }})
    }}

    async fn load_context(&self, args: &HashMap<String, String>) -> Result<PromptContext> {{
        // Load relevant context (files, config, docs, etc.)
        Ok(PromptContext {{
            // context fields
        }})
    }}

    fn build_system_message(&self, context: &PromptContext) -> String {{
        format!(
            "You are {{role description}}.\n\n\
             {{context information}}\n\n\
             {{guidelines}}"
        )
    }}

    fn build_user_message(
        &self,
        {arg1}: &str,
        {arg2}: &str,
        context: &PromptContext,
    ) -> String {{
        format!(
            "{{Task description with {arg1} and {arg2}}}\n\n\
             {{Additional instructions}}\n\n\
             {{Expected output format}}"
        )
    }}
}}

#[derive(Debug)]
struct PromptContext {{
    // Context fields
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_prompt_generation() {{
        let config = AppConfig::default();
        let prompt = {PromptName}Prompt::new(config);

        let mut args = HashMap::new();
        args.insert("{arg1}".to_string(), "test_value".to_string());

        let response = prompt.generate(args).await;
        assert!(response.is_ok());

        let response = response.unwrap();
        assert!(!response.messages.is_empty());
    }}

    #[tokio::test]
    async fn test_missing_required_argument() {{
        let config = AppConfig::default();
        let prompt = {PromptName}Prompt::new(config);

        let args = HashMap::new();
        let response = prompt.generate(args).await;
        assert!(response.is_err());
    }}
}}
```

### 3. Update prompts/mod.rs

```rust
pub mod {prompt_name};

pub use {prompt_name}::{{PromptName}Prompt;
```

### 4. Register Prompt in Service

Update `src/service.rs`:

```rust
use crate::prompts::{prompt_name}::{{PromptName}Prompt;

pub struct McpService {{
    {prompt_name}_prompt: {PromptName}Prompt,
    // ... other prompts
}}

impl McpService {{
    pub async fn new(config: AppConfig) -> Result<Self> {{
        Ok(Self {{
            {prompt_name}_prompt: {PromptName}Prompt::new(config.clone()),
            // ... other prompts
        }})
    }}

    /// List all available prompts
    pub fn list_prompts(&self) -> Vec<PromptInfo> {{
        vec![
            self.{prompt_name}_prompt.get_info(),
            // ... other prompts
        ]
    }}

    /// Get prompt by name with arguments
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: HashMap<String, String>,
    ) -> Result<PromptResponse> {{
        match name {{
            "{prompt_name}" => self.{prompt_name}_prompt.generate(arguments).await,
            _ => Err(Error::NotFound(format!("Prompt not found: {{}}", name))),
        }}
    }}
}}
```

## Prompt Templates

### Code Review Prompt

```rust
pub struct CodeReviewPrompt;

impl CodeReviewPrompt {{
    pub async fn generate(&self, args: HashMap<String, String>) -> Result<PromptResponse> {{
        let language = args.get("language").unwrap_or(&"any".to_string());
        let file_path = args.get("file_path");

        let system_msg = format!(
            "You are an expert code reviewer specializing in {{}} projects.\n\n\
             Your role is to:\n\
             - Identify bugs and logic errors\n\
             - Spot security vulnerabilities\n\
             - Suggest performance improvements\n\
             - Ensure code follows best practices\n\
             - Check for proper error handling\n\n\
             Provide specific, actionable feedback with line numbers.",
            language
        );

        let mut user_msg = String::from(
            "Please review this code carefully.\n\n\
             Focus on:\n\
             1. Correctness and bugs\n\
             2. Security issues\n\
             3. Performance concerns\n\
             4. Code style and maintainability\n\
             5. Test coverage\n\n"
        );

        if let Some(path) = file_path {{
            user_msg.push_str(&format!("File: {{}}\n\n", path));
        }}

        user_msg.push_str("Provide your review in this format:\n\
                           - Issue description\n\
                           - Severity (low/medium/high/critical)\n\
                           - Line number(s)\n\
                           - Suggested fix");

        Ok(PromptResponse {{
            description: Some("Code review prompt".to_string()),
            messages: vec![
                PromptMessage {{ role: Role::System, content: system_msg }},
                PromptMessage {{ role: Role::User, content: user_msg }},
            ],
        }})
    }}
}}
```

### Debug Session Prompt

```rust
pub struct DebugSessionPrompt;

impl DebugSessionPrompt {{
    pub async fn generate(&self, args: HashMap<String, String>) -> Result<PromptResponse> {{
        let language = args.get("language").unwrap_or(&"unknown".to_string());

        Ok(PromptResponse {{
            description: Some("Interactive debugging session".to_string()),
            messages: vec![
                PromptMessage {{
                    role: Role::System,
                    content: format!(
                        "You are an expert {{}} debugger. Help the user \
                         systematically identify and fix issues.",
                        language
                    ),
                }},
                PromptMessage {{
                    role: Role::Assistant,
                    content: "I'll help you debug this issue. Let's start with \
                              some questions:\n\n\
                              1. What did you expect to happen?\n\
                              2. What actually happened?\n\
                              3. What error messages did you see?\n\
                              4. What have you tried so far?\n\n\
                              Please share the relevant code and error output.".to_string(),
                }},
            ],
        }})
    }}
}}
```

### API Design Prompt

```rust
pub struct ApiDesignPrompt;

impl ApiDesignPrompt {{
    pub async fn generate(&self, args: HashMap<String, String>) -> Result<PromptResponse> {{
        let api_type = args.get("api_type").unwrap_or(&"REST".to_string());
        let domain = args.get("domain").unwrap();

        Ok(PromptResponse {{
            description: Some(format!("Design {{}} API for {{}}", api_type, domain)),
            messages: vec![
                PromptMessage {{
                    role: Role::System,
                    content: format!(
                        "You are an expert API architect with deep knowledge of {{}} APIs.\n\n\
                         Design principles:\n\
                         - Clear, intuitive endpoints\n\
                         - Proper use of HTTP methods\n\
                         - Consistent error handling\n\
                         - Versioning strategy\n\
                         - Security best practices\n\
                         - Documentation",
                        api_type
                    ),
                }},
                PromptMessage {{
                    role: Role::User,
                    content: format!(
                        "Design a {{}} API for a {{}} application.\n\n\
                         Please provide:\n\
                         1. Resource models and data structures\n\
                         2. Endpoint definitions with HTTP methods\n\
                         3. Request/response formats with examples\n\
                         4. Authentication and authorization approach\n\
                         5. Error handling strategy\n\
                         6. Rate limiting considerations\n\
                         7. Versioning plan\n\n\
                         Use industry best practices and standards.",
                        api_type, domain
                    ),
                }},
            ],
        }})
    }}
}}
```

### Learning Prompt

```rust
pub struct LearningPrompt;

impl LearningPrompt {{
    pub async fn generate(&self, args: HashMap<String, String>) -> Result<PromptResponse> {{
        let topic = args.get("topic").unwrap();
        let level = args.get("level").unwrap_or(&"beginner".to_string());

        Ok(PromptResponse {{
            description: Some(format!("Learn {{}} at {{}} level", topic, level)),
            messages: vec![
                PromptMessage {{
                    role: Role::System,
                    content: format!(
                        "You are a patient, knowledgeable teacher of {{}}.\n\n\
                         Teaching approach:\n\
                         - Start with fundamentals\n\
                         - Use clear, simple explanations\n\
                         - Provide practical examples\n\
                         - Check understanding regularly\n\
                         - Adapt to {{}} level\n\
                         - Encourage questions",
                        topic, level
                    ),
                }},
                PromptMessage {{
                    role: Role::User,
                    content: format!(
                        "I want to learn about {{}}. I'm at a {{}} level.\n\n\
                         Please:\n\
                         1. Explain the core concepts\n\
                         2. Show practical examples\n\
                         3. Build complexity gradually\n\
                         4. Provide exercises to practice\n\
                         5. Check my understanding\n\n\
                         Let's begin!",
                        topic, level
                    ),
                }},
            ],
        }})
    }}
}}
```

### Context-Rich Project Prompt

```rust
pub struct ProjectTaskPrompt {{
    context_loader: Arc<ContextLoader>,
}}

impl ProjectTaskPrompt {{
    pub async fn generate(&self, args: HashMap<String, String>) -> Result<PromptResponse> {{
        let task = args.get("task").unwrap();

        // Load project context
        let project = self.context_loader.load_project_info().await?;
        let dependencies = self.context_loader.load_dependencies().await?;
        let patterns = self.context_loader.load_code_patterns().await?;

        let context = format!(
            "# Project Context\n\n\
             **Project:** {{}}\n\
             **Language:** {{}}\n\
             **Framework:** {{}}\n\n\
             # Dependencies\n\n{{}}\n\n\
             # Code Patterns\n\n{{}}\n\n",
            project.name,
            project.language,
            project.framework,
            dependencies.join(", "),
            patterns
        );

        Ok(PromptResponse {{
            description: Some(format!("Project task: {{}}", task)),
            messages: vec![
                PromptMessage {{
                    role: Role::User,
                    content: format!(
                        "{{}}\
                         # Task\n\n{{}}\n\n\
                         # Guidelines\n\n\
                         - Follow existing project patterns\n\
                         - Maintain code style consistency\n\
                         - Update tests and documentation\n\
                         - Consider existing dependencies",
                        context, task
                    ),
                }},
            ],
        }})
    }}
}}
```

## After Creation

```
✅ Prompt '{prompt_name}' added successfully!

## Files Created/Modified:
- src/prompts/{prompt_name}.rs - Prompt implementation
- src/prompts/mod.rs - Module export
- src/service.rs - Prompt registration

## Next Steps:

1. **Customize the prompt:**
   Edit src/prompts/{prompt_name}.rs

2. **Test prompt:**
   ```bash
   cargo test {prompt_name}
   ```

3. **Use in MCP client:**
   List prompts, then get the prompt with arguments

## Example Usage:

List prompts:
\```json
{{"jsonrpc":"2.0","method":"prompts/list","params":{{}},"id":1}}
\```

Get prompt:
\```json
{{
  "jsonrpc":"2.0",
  "method":"prompts/get",
  "params":{{
    "name":"{prompt_name}",
    "arguments":{{{{"arg1":"value1"}}}}
  }},
  "id":2
}}
\```
```
