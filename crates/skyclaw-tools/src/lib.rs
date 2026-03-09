//! SkyClaw Tools — agent capabilities (shell, file, web, browser, etc.)

#[cfg(feature = "browser")]
mod browser;
mod check_messages;
mod file;
mod git;
mod key_manage;
mod memory_manage;
mod send_file;
mod send_message;
mod shell;
mod web_fetch;

#[cfg(feature = "browser")]
pub use browser::BrowserTool;
pub use check_messages::{CheckMessagesTool, PendingMessages};
pub use file::{FileListTool, FileReadTool, FileWriteTool};
pub use git::GitTool;
pub use key_manage::KeyManageTool;
pub use memory_manage::MemoryManageTool;
pub use send_file::SendFileTool;
pub use send_message::SendMessageTool;
pub use shell::ShellTool;
pub use web_fetch::WebFetchTool;

use skyclaw_core::types::config::ToolsConfig;
use skyclaw_core::{Channel, Memory, SetupLinkGenerator, Tool};
use std::sync::Arc;

/// Create tools based on the configuration flags.
/// Pass an optional channel for file transfer tools, an optional
/// pending-message queue for the check_messages tool, an optional
/// memory backend for the memory_manage tool, and an optional
/// setup link generator for the key_manage tool.
pub fn create_tools(
    config: &ToolsConfig,
    channel: Option<Arc<dyn Channel>>,
    pending_messages: Option<PendingMessages>,
    memory: Option<Arc<dyn Memory>>,
    setup_link_gen: Option<Arc<dyn SetupLinkGenerator>>,
) -> Vec<Arc<dyn Tool>> {
    let mut tools: Vec<Arc<dyn Tool>> = Vec::new();

    if config.shell {
        tools.push(Arc::new(ShellTool::new()));
    }

    if config.file {
        tools.push(Arc::new(FileReadTool::new()));
        tools.push(Arc::new(FileWriteTool::new()));
        tools.push(Arc::new(FileListTool::new()));
    }

    if config.git {
        tools.push(Arc::new(GitTool::new()));
    }

    if config.http {
        tools.push(Arc::new(WebFetchTool::new()));
    }

    // Add channel-dependent tools
    if let Some(ch) = channel {
        // send_message: send intermediate text messages during tool execution
        tools.push(Arc::new(SendMessageTool::new(ch.clone())));

        // send_file: send files if channel supports file transfer
        if ch.file_transfer().is_some() {
            tools.push(Arc::new(SendFileTool::new(ch)));
        }
    }

    // check_messages: lets agent peek at pending user messages during tasks
    if let Some(pending) = pending_messages {
        tools.push(Arc::new(CheckMessagesTool::new(pending)));
    }

    // memory_manage: persistent knowledge store for the agent
    if let Some(mem) = memory {
        tools.push(Arc::new(MemoryManageTool::new(mem)));
    }

    // key_manage: generates setup links and guides users through key operations
    tools.push(Arc::new(KeyManageTool::new(setup_link_gen)));

    // browser: headless Chrome automation (stealth mode)
    #[cfg(feature = "browser")]
    if config.browser {
        tools.push(Arc::new(BrowserTool::with_timeout(
            config.browser_timeout_secs,
        )));
    }

    tracing::info!(count = tools.len(), "Tools registered");
    tools
}
