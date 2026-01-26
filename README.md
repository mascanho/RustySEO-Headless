# 🦀 RustySEO CLI

<div align="center">

![RustySEO Logo](https://github.com/mascanho/RustySEO/raw/main/assets/icon.png)

**A powerful, terminal-based SEO analysis tool built with Rust**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange.svg)](https://crates.io/crates/rustyseo-cli)

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Screenshots](#-screenshots) • [Contributing](#-contributing)


![RustySEO Terminal](https://github.com/mascanho/RustySEO/raw/main/assets/tui.webp)
</div>

---

## 🚀 About RustySEO

RustySEO CLI is a cutting-edge, terminal-based SEO analysis tool that combines the power of Rust with modern web crawling technologies. It provides comprehensive website audits, technical SEO analysis, and actionable insights—all within a beautiful, intuitive terminal interface.

### ✨ Why RustySEO?

- **⚡ Blazing Fast**: Built with Rust for maximum performance and reliability
- **🎯 Comprehensive Analysis**: Detects 15+ types of SEO issues automatically
- **🌐 Modern Crawling**: JavaScript-enabled crawling with real browser rendering
- **🤖 AI-Powered Insights**: Integrated AI chat for SEO recommendations
- **📊 Rich Visualizations**: Beautiful terminal charts and data displays
- **⚙️ Highly Configurable**: Extensive settings and customization options

---

## 🎯 Features

### 🔍 **Technical SEO Analysis**
- **HTTP Status Analysis**: 4xx, 5xx, and 3xx redirect detection
- **Content Optimization**: Title length, meta description analysis
- **HTML Structure**: H1 tags, alt text, semantic HTML validation
- **Performance Metrics**: Core Web Vitals and page speed insights
- **Link Analysis**: Internal/external link discovery and validation

### 🌐 **Advanced Web Crawling**
- **JavaScript Rendering**: Full browser-based crawling with Headless Chrome
- **Concurrent Processing**: Multi-threaded crawling for maximum speed
- **Smart Discovery**: Sitemap and robots.txt parsing
- **User Agent Rotation**: Realistic browser simulation
- **Rate Limiting**: Respectful crawling with configurable delays

### 📊 **Data Visualization**
- **Interactive Tables**: Sortable, filterable data displays
- **Terminal Charts**: Pie charts, bar graphs, and progress indicators
- **Color-Coded Issues**: Visual highlighting of problems
- **Real-time Progress**: Live crawling statistics and updates

### 🤖 **AI Integration**
- **Smart Recommendations**: AI-powered SEO suggestions
- **Natural Language Chat**: Ask questions about your SEO data
- **Multiple Providers**: OpenAI, Gemini, and custom AI endpoints
- **Context-Aware**: AI understands your specific website data

### ⚙️ **Configuration & Settings**
- **Persistent Settings**: TOML-based configuration files
- **Customizable Crawling**: Depth limits, concurrency, timeouts
- **Export Options**: Data export in multiple formats
- **Bookmark Management**: Save and organize frequently analyzed sites

---

## 📦 Installation

### 🦀 From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/rustyseo-cli.git
cd rustyseo-cli

# Build and install
cargo install --path .

# Or run directly
cargo run
```

### 📦 System Requirements

- **Rust 1.70+** (for building from source)
- **Node.js 16+** (for JavaScript crawling features)
- **Chrome/Chromium** (for headless browser rendering)
- **Linux/macOS/Windows** support

---

## 🎮 Usage

### Quick Start

```bash
# Interactive mode (default)
rustyseo

# Direct URL analysis
rustyseo -u https://example.com

# Help
rustyseo --help
```

### 🎯 Interactive Mode

Launch RustySEO and use the intuitive interface:

1. **Enter URL**: Type your website URL and press Enter
2. **Watch Crawl**: Real-time crawling progress and statistics
3. **Analyze Results**: Navigate through different analysis tabs
4. **Get Insights**: Use AI chat for personalized recommendations

### ⌨️ Keyboard Shortcuts

| Category | Shortcut | Action |
|----------|----------|--------|
| **Navigation** | `Tab` / `Shift+Tab` | Switch between tabs |
| | `1-9` / `0` | Jump to specific tab |
| | `↑/↓` / `j/k` | Navigate lists |
| | `Enter` | Select item / open details |
| | `Esc` | Close modal / go back |
| **Search** | `Ctrl+F` | Open search |
| | `Enter` | Apply filter |
| | `Esc` | Close search |
| **Actions** | `m` | Open actions menu |
| | `o` | Open URL in browser |
| | `c` | Copy URL to clipboard |
| **Interface** | `g` | Open sidebar |
| | `i` | Issues tab |
| | `b` | Bookmarks tab |
| | `t` | Tree view |
| | `s` | Settings tab |
| | `L` | Toggle logs |
| | `?` | Show help |
| | `q` | Quit application |

---

## 📸 Screenshots

### 🎯 Main Dashboard
![Main Dashboard](https://raw.githubusercontent.com/user-attachments/assets/placeholder-dashboard.png)
*Comprehensive SEO analysis dashboard with real-time statistics*

### 🔍 Issues Analysis
![Issues Analysis](https://raw.githubusercontent.com/user-attachments/assets/placeholder-issues.png)
*Color-coded issue detection with detailed URL listings*

### 🤖 AI Chat Interface
![AI Chat](https://raw.githubusercontent.com/user-attachments/assets/placeholder-ai-chat.png)
*AI-powered SEO recommendations and insights*

### 📊 Data Visualization
![Data Charts](https://raw.githubusercontent.com/user-attachments/assets/placeholder-charts.png)
*Interactive charts and progress indicators*

---

## 🔧 Configuration

RustySEO uses a TOML-based configuration system. Settings are automatically created in:

- **Linux/macOS**: `~/.config/rustyseo/settings.toml`
- **Windows**: `%APPDATA%\rustyseo\settings.toml`

### Example Configuration

```toml
[crawler]
max_pages = 100
concurrency_limit = 10
enable_javascript = true
delay_ms = 1000

[connectors]
selected = "openai"

[connectors.openai]
model = "gpt-4"
api_key = "your-api-key-here"

[ui]
theme = "dark"
show_logs = true
auto_scroll = true
```

---

## 📋 Analysis Types

RustySEO automatically detects and reports on these SEO issues:

### 🚨 **Critical Issues**
- **4xx Errors**: Broken links and missing pages
- **5xx Errors**: Server errors and downtime
- **Missing H1**: No primary heading tags

### ⚠️ **Content Issues**
- **Title Length**: Too long (>60 chars) or too short (<30 chars)
- **Meta Descriptions**: Missing or too long (>160 chars)
- **Missing Alt Text**: Images without accessibility descriptions

### 🔄 **Technical Issues**
- **3XX Redirects**: Redirect chains and loops
- **Internal Links**: Broken internal navigation
- **External Links**: Dead outbound links

### 📈 **Performance Metrics**
- **Core Web Vitals**: LCP, FID, CLS measurements
- **Page Speed**: Load time optimization suggestions
- **Resource Analysis**: Image and script optimization

---

## 🤖 AI Integration

RustySEO supports multiple AI providers for intelligent SEO recommendations:

### 🧠 **Supported Providers**
- **OpenAI GPT-4**: Advanced reasoning and analysis
- **Google Gemini**: Fast and efficient insights
- **Custom Endpoints**: Bring your own AI service

### 💬 **AI Chat Features**
- **Context-Aware**: AI understands your specific website data
- **Actionable Advice**: Get specific recommendations for each issue
- **Natural Language**: Ask questions in plain English
- **Multi-Language**: Support for various languages and regions

---

## 🛠️ Advanced Features

### 🌳 **Tree View Navigation**
- Hierarchical website structure visualization
- Interactive folder/file exploration
- Quick navigation to specific pages

### 📚 **Bookmark Management**
- Save frequently analyzed websites
- Organize by category or project
- Quick access from sidebar

### 📊 **Export Capabilities**
- CSV export for spreadsheet analysis
- JSON export for programmatic use
- PDF reports for client presentations

### 🔍 **Search & Filtering**
- Real-time search across all data
- Fuzzy matching for flexible queries
- Filter by issue type or severity

---

## 🚧 Development

### 🏗️ Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rustyseo-cli.git
cd rustyseo-cli

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run
```

### 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test crawler

# Run with coverage
cargo tarpaulin --out Html
```

### 📝 Code Style

This project uses:
- **rustfmt** for code formatting
- **clippy** for linting
- **pre-commit hooks** for quality assurance

---

## 🤝 Contributing

We welcome contributions! Here's how to get started:

### 🌟 **Ways to Contribute**

1. **🐛 Report Bugs**: Open an issue with detailed information
2. **💡 Feature Requests**: Suggest new features or improvements
3. **📝 Documentation**: Help improve README and code comments
4. **🔧 Code Contributions**: Fix bugs or implement new features
5. **🧪 Testing**: Write tests and improve coverage

### 📋 **Getting Started**

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### 📖 **Guidelines**

- Follow the existing code style
- Add tests for new features
- Update documentation as needed
- Ensure all tests pass before submitting

---

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## ⚠️ Disclaimer

**🚨 IMPORTANT NOTICE**

RustySEO CLI is provided as-is for educational and analysis purposes. Users are responsible for:

- **Compliance**: Ensure you have permission to analyze websites
- **Rate Limiting**: Respect website robots.txt and rate limits
- **Data Privacy**: Handle sensitive data according to applicable laws
- **Commercial Use**: Check licensing for commercial applications

**⚠️ NO WARRANTY**: The software is provided "as is" without warranty of any kind. The authors are not liable for any damages arising from the use of this software.

**🔒 Privacy**: No data is sent to external servers except for configured AI services. All crawling and analysis happens locally on your machine.

---

## 🙏 Acknowledgments

- **Rust Community**: For the amazing language and ecosystem
- **Ratatui**: For the excellent terminal UI framework
- **Headless Chrome**: For powerful browser automation
- **All Contributors**: Who help make this project better

---

## 📞 Support & Community

- **🐛 Issues**: [GitHub Issues](https://github.com/yourusername/rustyseo-cli/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/yourusername/rustyseo-cli/discussions)
- **📖 Wiki**: [Project Wiki](https://github.com/yourusername/rustyseo-cli/wiki)
- **🐦 Twitter**: [@RustySEO_CLI](https://twitter.com/RustySEO_CLI)

---

<div align="center">

**⭐ Star this repo if you find it useful!**

Made with ❤️ and 🦀 by the RustySEO Team

[Back to Top](#-rustyseo-cli)

</div>
