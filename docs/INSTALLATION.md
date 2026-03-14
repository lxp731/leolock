## 📦 安装

### Debian/Ubuntu/RHEL/Fedora

```bash
# 在仓库下载 .deb 包或者 .rpm 包
```

### AUR

```bash
yay -S leolock
```

### 从源码编译

```bash
# 克隆项目
git clone https://github.com/lxp731/leolock.git
cd leolock

# 编译发布版本
cargo build --release

# 安装到系统（可选）
sudo cp target/release/leolock /usr/local/bin/
```

### 生成 Tab 补全

LeoLock 支持 5 种 shell 的自动补全：

```bash
# 生成所有支持的补全脚本到当前目录
leolock completions bash
leolock completions zsh
leolock completions fish
leolock completions powershell
leolock completions elvish

# 或指定输出目录
leolock completions bash -o ~/.bash_completion.d/
```

#### 安装补全脚本

**Bash**:
```bash
# 系统级安装
sudo leolock completions bash -o /usr/share/bash-completion/completions/

# 用户级安装
mkdir -p ~/.bash_completion.d
leolock completions bash -o ~/.bash_completion.d/
echo "source ~/.bash_completion.d/leolock.bash" >> ~/.bashrc
source ~/.bashrc
```

**Zsh**:
```bash
# 系统级安装
sudo leolock completions zsh -o /usr/share/zsh/site-functions/

# 用户级安装
mkdir -p ~/.zsh/completions
leolock completions zsh -o ~/.zsh/completions/
echo "fpath=(~/.zsh/completions \$fpath)" >> ~/.zshrc
echo "autoload -Uz compinit && compinit" >> ~/.zshrc
source ~/.zshrc
```

**Fish**:
```bash
# 系统级安装
sudo leolock completions fish -o /usr/share/fish/vendor_completions.d/

# 用户级安装
mkdir -p ~/.config/fish/completions
leolock completions fish -o ~/.config/fish/completions/
```

**PowerShell**:
```bash
# 生成补全脚本
leolock completions powershell -o ~/.config/powershell/

# 在 PowerShell 配置文件中添加
# Add-Content -Path $PROFILE -Value ". ~/.config/powershell/_leolock.ps1"
```

**Elvish**:
```bash
# 生成补全脚本
leolock completions elvish -o ~/.config/elvish/lib/

# 在 Elvish 配置文件中添加
# use leolock = ~/.config/elvish/lib/leolock.elv
```

