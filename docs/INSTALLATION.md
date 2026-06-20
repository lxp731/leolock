# LeoLock 安装指南

## 一键安装（推荐）

```bash
curl -fsSL https://raw.githubusercontent.com/lxp731/leolock/main/install.sh | bash
```

自动检测系统架构（Linux/macOS, x86_64/ARM），从 GitHub Releases 下载最新版本。

**指定版本:**
```bash
LEOLOCK_VERSION=1.2.0 bash install.sh
```

**指定安装目录:**
```bash
INSTALL_DIR=~/.local/bin bash install.sh
```

## 包管理器安装

### Debian/Ubuntu (.deb)
```bash
curl -fsSL https://github.com/lxp731/leolock/releases/latest/download/leolock_amd64.deb -o /tmp/leolock.deb
sudo dpkg -i /tmp/leolock.deb
```

### RHEL/Fedora (.rpm)
```bash
sudo rpm -i https://github.com/lxp731/leolock/releases/latest/download/leolock.x86_64.rpm
```

### Arch Linux (AUR)
```bash
yay -S leolock
```

### 从 Git 仓库
```bash
cargo install --git https://github.com/lxp731/leolock.git
```

## 源码编译

```bash
git clone https://github.com/lxp731/leolock.git
cd leolock

# 编译发布版本（LTO + 最高优化）
cargo build --release

# 安装到系统目录
sudo cp target/release/leolock /usr/local/bin/

# 或安装到用户目录
cp target/release/leolock ~/.local/bin/
```

## 验证安装

```bash
leolock --version
```

## Shell 补全

LeoLock 支持 Bash、Zsh、Fish、PowerShell、Elvish：

```bash
# 生成补全脚本
leolock completions bash   -o ~/.bash_completion.d/
leolock completions zsh    -o ~/.zsh/completions/
leolock completions fish   -o ~/.config/fish/completions/

# 系统级安装
sudo leolock completions bash -o /usr/share/bash-completion/completions/
sudo leolock completions zsh  -o /usr/share/zsh/site-functions/
```
