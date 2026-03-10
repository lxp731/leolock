# LeoLock 发布指南

## 发布流程

### 1. 更新版本号
```bash
# 编辑 Cargo.toml 更新版本号
vim Cargo.toml
# version = "1.0.0" → version = "1.0.1"

# 提交更改
git add Cargo.toml
git commit -m "Bump version to 1.0.1"
```

### 2. 创建 Git 标签
```bash
# 创建带注释的标签
git tag -a v1.0.1 -m "Release version 1.0.1"

# 推送标签到 GitHub
git push origin v1.0.1
```

### 3. 自动触发的工作流

推送标签后会自动触发两个工作流：

#### 工作流 1: `Source Release` (release.yml)
- 创建源码包 (`leolock-1.0.1.tar.gz`)
- 创建 Linux 二进制包 (`leolock-1.0.1-x86_64-linux-gnu.tar.gz`)
- 生成 SHA256 校验和
- 创建 GitHub Release 并上传文件

#### 工作流 2: `Build Packages` (build-packages.yml)
- 构建 `.deb` 包 (Debian/Ubuntu)
- 构建 `.rpm` 包 (Fedora/RHEL/CentOS)
- 支持 x86_64 和 aarch64 架构
- 将包文件添加到已有的 GitHub Release

### 4. 手动触发（可选）
如果自动触发失败，可以手动触发：

1. 访问 GitHub Actions 页面
2. 选择对应的工作流
3. 点击 "Run workflow"
4. 选择分支和版本标签

## 生成的文件

### 源码发布
```
leolock-1.0.1.tar.gz              # 完整源代码
leolock-1.0.1-x86_64-linux-gnu.tar.gz  # Linux 二进制
SHA256SUMS                        # 校验和文件
```

### 包管理器发布
```
leolock_1.0.1-1_amd64.deb         # Debian/Ubuntu 包
leolock-1.0.1-1.x86_64.rpm        # Fedora/RHEL/CentOS 包
```

## 安装方法

### 从源码安装
```bash
# 下载源码
wget https://github.com/lxp731/leolock/releases/download/v1.0.1/leolock-1.0.1.tar.gz
tar -xzf leolock-1.0.1.tar.gz
cd leolock-1.0.1

# 构建
cargo build --release

# 安装
sudo cp target/release/leolock /usr/local/bin/
```

### 使用二进制文件
```bash
wget https://github.com/lxp731/leolock/releases/download/v1.0.1/leolock-1.0.1-x86_64-linux-gnu.tar.gz
tar -xzf leolock-1.0.1-x86_64-linux-gnu.tar.gz
chmod +x leolock
sudo mv leolock /usr/local/bin/
```

### 使用 .deb 包 (Debian/Ubuntu)
```bash
wget https://github.com/lxp731/leolock/releases/download/v1.0.1/leolock_1.0.1-1_amd64.deb
sudo dpkg -i leolock_1.0.1-1_amd64.deb
# 如果依赖问题
sudo apt --fix-broken install
```

### 使用 .rpm 包 (Fedora/RHEL/CentOS)
```bash
wget https://github.com/lxp731/leolock/releases/download/v1.0.1/leolock-1.0.1-1.x86_64.rpm
sudo rpm -i leolock-1.0.1-1.x86_64.rpm
# 或使用 dnf
sudo dnf install ./leolock-1.0.1-1.x86_64.rpm
```

## 验证下载
```bash
# 下载校验和文件
wget https://github.com/lxp731/leolock/releases/download/v1.0.1/SHA256SUMS

# 验证文件
sha256sum -c SHA256SUMS
```

## 故障排除

### 工作流失败
1. 检查 Rust 工具链版本
2. 确保 Cargo.toml 中的版本号正确
3. 检查依赖包是否可用

### 包构建失败
1. 确保已安装 `cargo-deb` 和 `cargo-generate-rpm`
2. 检查系统依赖是否满足
3. 查看构建日志中的具体错误

### 发布失败
1. 确保 GitHub Token 有足够权限
2. 检查网络连接
3. 验证标签名称格式正确

## 更新 AUR 包

发布到 GitHub 后，记得更新 AUR 包：

```bash
cd ~/aur/leolock
# 更新 PKGBUILD 中的版本号和哈希值
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to v1.0.1"
git push origin master
```

## 联系方式

如有问题，请：
1. 查看 GitHub Actions 日志
2. 提交 Issue
3. 联系维护者