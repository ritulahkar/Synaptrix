# Synaptrix

A fast, lightweight, and modern application launcher for Linux, inspired by Synapse but built from the ground up in Rust.

## 🚀 About

Synaptrix is a GPL3-licensed application launcher designed specifically for Linux Mint (and other Linux distributions). Born out of the need for a fast, reliable, and actively maintained launcher that captures the simplicity and efficiency of Synapse, Synaptrix leverages Rust's performance and safety to deliver a snappy user experience.

## ✨ Features

- **Lightning Fast**: Built in Rust for maximum performance and minimal resource usage
- **Synapse-like Interface**: Familiar workflow for users coming from Synapse
- **Modern Architecture**: Clean, maintainable codebase designed for long-term sustainability
- **Linux Mint Optimized**: Thoroughly tested and optimized for Linux Mint environments
- **Open Source**: Fully open source under GPL3 license

## 🎯 Why Synaptrix?

The Linux desktop ecosystem was missing a fast, Rust-based launcher that:
- Actually works reliably
- Is actively maintained
- Provides the smooth Synapse-like experience users love
- Takes advantage of modern programming practices

Synaptrix fills this gap by combining the best aspects of traditional launchers with modern Rust performance.

## 🛠️ Installation

### From Source
```bash
git clone https://github.com/ritulahkar/Synaptrix.git
cd synaptrix
cargo build --release
sudo cp target/release/synaptrix /usr/local/bin/
```

### Dependencies
- Rust 1.70 or later
- GTK development libraries
- Linux Mint 20+ (or compatible distributions)

## 🚀 Usage

Launch Synaptrix by running:
```bash
synaptrix
```

Or set up a keyboard shortcut in your system settings to launch it with a hotkey (recommended: `Ctrl+Space`).

## 🤝 Contributing

**Important Note**: I'm a self-taught programmer who builds projects with the help of AI tools. This means:

- The codebase might not follow all best practices initially
- There's definitely room for improvement in architecture and implementation
- **Your contributions are not just welcome—they're essential!**

### How You Can Help

- **Code Reviews**: Help improve code quality and Rust best practices
- **Bug Reports**: Found an issue? Please report it!
- **Feature Requests**: Have ideas for new features? Let's discuss them!
- **Documentation**: Help improve documentation and user guides
- **Testing**: Test on different Linux distributions and hardware
- **Refactoring**: Help modernize and optimize the codebase

### Getting Started with Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Test thoroughly
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

Don't worry if you're new to Rust or open source—everyone starts somewhere! Feel free to ask questions in issues or discussions.

## 🐛 Known Issues

- [List any known issues here]
- Performance optimization ongoing
- Feature parity with Synapse still in development

## 📋 Roadmap

- [ ] Complete feature parity with Synapse
- [ ] Plugin system for extensibility
- [ ] Themes and customization options
- [ ] Multi-monitor support improvements
- [ ] Wayland compatibility
- [ ] Package manager integration

## 🔧 Development

### Building from Source
```bash
git clone https://github.com/[your-username]/synaptrix.git
cd synaptrix
cargo build
```

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## 📝 License

This project is licensed under the GPL3 License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by the original Synapse launcher
- Built with the Rust programming language
- Developed with assistance from AI tools
- Thanks to the Linux Mint and broader Linux community

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/[your-username]/synaptrix/issues)
- **Discussions**: [GitHub Discussions](https://github.com/[your-username]/synaptrix/discussions)
- **Community**: Join our discussions and help shape the future of Synaptrix!

## 🌟 Star History

If you find Synaptrix useful, please consider giving it a star! It helps others discover the project and motivates continued development.

---

**Note**: This project is a community effort. As a learning developer, I rely on the expertise and contributions of the community to make Synaptrix the best launcher it can be. Every contribution, no matter how small, makes a difference!
