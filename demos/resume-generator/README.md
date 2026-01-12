# Resume Generator - Document Generation Demo

A demonstration of Claude Agent SDK's web search and document generation capabilities, migrated from original TypeScript implementation.

## Purpose

This demo showcases advanced SDK features:
- **Web Search Integration**: Researching people and gathering information
- **AI-Assisted Document Generation**: Creating professional resumes
- **Template-Based Formatting**: Structured resume sections
- **Focus Area Customization**: Tailoring content to specific skills

## Original Implementation

Migrated from: `claude-agent-sdk-demos/resume-generator/resume-generator.ts`

### Key Differences

| TypeScript | Rust |
|-----------|------|
| `query()` with `prompt` | `ClaudeAgentClient::query()` |
| `allowedTools` array | `allowed_tools: Vec<String>` |
| `systemPrompt` string | `system_prompt: Option<String>` |
| `maxTurns` number | `max_turns: Option<u32>` |
| `settingSources: ['project']` | Not needed in Rust SDK |

### Simplifications

The Rust version simplifies some aspects of the original TypeScript demo:

1. **Skill System**: The original used `settingSources: ['project']` to load skills from `.claude/skills/`. The Rust version demonstrates the pattern without requiring skill files.

2. **Docx Library**: The original used a `docx` skill for Word document generation. The Rust version shows the pattern for document creation, with the actual .docx generation left as an exercise.

3. **File Output**: The original saved to `agent/custom_scripts/resume.docx`. The Rust version demonstrates the workflow with file output tracking.

## Prerequisites

- Rust 1.75+
- Claude Code CLI installed and authenticated
- ANTHROPIC_API_KEY environment variable set

## Setup

```bash
# From claude-agent-rust directory
cd demos/resume-generator

# Run the demo
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

## Usage

### Basic Usage

Generate a resume for a person:

```bash
$ cargo run -- "Jane Doe"

Claude Agent SDK - Resume Generator Demo
Demonstrating web search and document generation

Sending query: Research 'Jane Doe' and create a professional 1-page resume...
--------------------------------------------------
Researching Jane Doe's background...
Searching for professional information...
Creating resume with sections:
- Professional Summary
- Experience
- Education
- Skills
- Projects
--------------------------------------------------
Resume generation completed!
Tools used: WebSearch, WebFetch, Write
Response length: 1247 characters
```

### With Focus Areas

Generate a resume focusing on specific skills:

```bash
$ cargo run -- "John Smith" --focus leadership,management

Generating resume with focus on: leadership, management
--------------------------------------------------
Researching John Smith's leadership experience...
Highlighting management achievements...
--------------------------------------------------
Resume generation completed!
```

### Command-Line Options

```bash
cargo run -- <person_name> [--focus <area1>,<area2>]

# Arguments:
#   person_name   - Name of person to research (required)
#   --focus       - Comma-separated list of focus areas (optional)
```

### Examples

```bash
# Basic resume generation
cargo run -- "Alice Johnson"

# Focus on technical skills
cargo run -- "Bob Williams" --focus programming,development

# Focus on leadership
cargo run -- "Carol Davis" --focus leadership,management,strategy

# Multiple focus areas
cargo run -- "David Lee" --focus sales,customer-service,communication
```

## Features Demonstrated

### 1. Web Search Integration

```rust
let options = ClaudeAgentOptions {
    allowed_tools: vec![
        "WebSearch".to_string(),
        "WebFetch".to_string(),
        "Write".to_string(),
        "Read".to_string(),
    ],
    ..Default::default()
};
```

### 2. Document Generation Workflow

```rust
let prompt = format!(
    "Research '{}' and create a professional 1-page resume in .docx format.",
    person_name
);

let mut stream = client.query(&prompt).await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(Message::Assistant(msg)) => {
            for block in msg.content {
                if let ContentBlock::ToolUse(tool_use) = block {
                    println!("Tool used: {}", tool_use.name);
                }
                if let ContentBlock::Text(text) = block {
                    println!("{}", text.text);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### 3. Template-Based Formatting

```rust
let system_prompt = "You are a professional resume writer. Research the person and create a 1-page resume in .docx format. \
     Include sections for: Professional Summary, Experience, Education, Skills, and Projects. \
     Use professional formatting with clear headings and bullet points. \
     Focus on achievements and quantifiable results. \
     Keep it concise and impactful.";
```

### 4. Focus Area Customization

```rust
let focus_areas = vec!["leadership", "management"];

let focus_prompt = format!(
    "Research '{}' and create a professional resume focusing on: {}. \
     Include relevant achievements and quantifiable results in these areas.",
    person_name,
    focus_areas.join(", ")
);
```

## Architecture

```
resume-generator/
├── Cargo.toml              # Project dependencies
├── src/
│   └── main.rs             # Main application
├── tests/
│   └── integration_test.rs  # Comprehensive test suite
└── README.md               # This file
```

### Component Overview

**Main Application (`main.rs`)**:
- `generate_resume()` - Basic resume generation
- `generate_resume_with_focus()` - Resume with focus areas
- Command-line argument parsing
- Tool usage tracking

**Test Suite (`tests/integration_test.rs`)**:
- `test_resume_generation_basic` - Basic workflow
- `test_resume_with_focus_areas` - Focus customization
- `test_tool_whitelisting` - Tool access control
- `test_max_turns_enforcement` - Turn limit
- `test_system_prompt_application` - Resume writing context
- `test_error_handling` - Connection failures
- `test_disconnect_cleanup` - Resource management
- `test_model_selection` - Model configuration

## Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_resume_generation_basic

# Run tests with logging
RUST_LOG=debug cargo test
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage/

# View report
open coverage/index.html
```

### Test Cases

| Test | Description | Status |
|-------|-------------|----------|
| `test_resume_generation_basic` | Basic workflow | ✅ |
| `test_resume_with_focus_areas` | Focus customization | ✅ |
| `test_tool_whitelisting` | Tool access control | ✅ |
| `test_max_turns_enforcement` | Turn limit | ✅ |
| `test_system_prompt_application` | Resume writing context | ✅ |
| `test_error_handling` | Connection failures | ✅ |
| `test_disconnect_cleanup` | Resource management | ✅ |
| `test_model_selection` | Model configuration | ✅ |

**Coverage Target:** ≥80%

## Resume Structure

The generated resume includes these sections:

1. **Professional Summary**
   - 2-3 sentence overview
   - Key strengths and value proposition

2. **Experience**
   - Work history in reverse chronological order
   - Job titles, companies, dates
   - Achievements with quantifiable results

3. **Education**
   - Degrees, institutions, graduation years
   - Relevant coursework or honors

4. **Skills**
   - Technical and soft skills
   - Proficiency levels (optional)

5. **Projects**
   - Notable projects or achievements
   - Technologies used
   - Impact and results

## Migration Notes

### Differences from Original TypeScript

**Simplifications:**
1. **Skill System**: The original used `.claude/skills/docx/` for Word document generation. The Rust version demonstrates the pattern without requiring skill files.

2. **File Output**: The original saved to `agent/custom_scripts/resume.docx`. The Rust version shows the workflow with file output tracking.

3. **Web Search Results**: The original expected actual web search results. The Rust version uses mock responses for testing.

### Enhancements

1. **Strong Typing**: Rust's type system ensures correct message handling at compile time.

2. **Error Context**: Using `anyhow::Context` provides better error messages.

3. **Structured Logging**: Integration with `tracing` for structured, filterable logs.

4. **Focus Areas**: Added command-line option for customizing resume focus.

5. **Tool Tracking**: Displays which tools were used during generation.

### Known Limitations

1. **Actual .docx Generation**: The demo shows the pattern for document creation, but actual .docx file generation requires additional libraries (e.g., `docx` crate).

2. **Real Web Search**: The demo uses the SDK's web search capabilities, but actual results depend on API access.

3. **File Persistence**: The demo doesn't persist generated resumes to disk (left as exercise).

## Troubleshooting

### Connection Errors

```
Error: Failed to connect to Claude Code
```

**Solution**: Ensure Claude Code CLI is installed and authenticated:
```bash
claude --version
claude login
```

### API Key Errors

```
Error: ANTHROPIC_API_KEY not found
```

**Solution**: Set environment variable:
```bash
export ANTHROPIC_API_KEY=your_key_here
```

Or create a `.env` file:
```
ANTHROPIC_API_KEY=your_key_here
```

### Build Errors

```
Error: failed to run custom build command
```

**Solution**: Ensure Rust 1.75+ is installed:
```bash
rustc --version
rustup update
```

### Web Search Failures

```
Tool used: WebSearch
Error: Web search failed
```

**Solution**: Check API access and network connectivity:
```bash
# Test network connectivity
ping api.anthropic.com

# Verify API key
echo $ANTHROPIC_API_KEY
```

## Next Steps

After mastering this demo, explore:

- **Research Agent**: Multi-agent orchestration and complex workflows
- **Simple Chat App**: Real-time WebSocket communication
- **Email Agent**: IMAP integration and email processing

## Extending the Demo

### Adding .docx Generation

To generate actual Word documents, add the `docx` crate:

```toml
# Cargo.toml
[dependencies]
docx = "0.2"
```

```rust
use docx::Docx;

let mut docx_file = Docx::new();
// Add content...
docx_file.save("resume.docx")?;
```

### Adding More Resume Sections

Customize the system prompt to include additional sections:

```rust
let system_prompt = "You are a professional resume writer. Include sections for: \
     Professional Summary, Experience, Education, Skills, Projects, \
     Certifications, Publications, and Languages.";
```

### Adding Resume Templates

Create different resume templates for different industries:

```rust
let templates = HashMap::from([
    ("tech", "Technology resume template..."),
    ("finance", "Finance resume template..."),
    ("creative", "Creative resume template..."),
]);

let template = templates.get(industry).unwrap_or(&default_template);
```

## License

MIT OR Apache-2.0
