<h1 align="center">🧰Steamtools</h1>


<p align="center">
  <img src="https://img.shields.io/github/v/tag/RealViper8/steamtools" />
  <img src="https://img.shields.io/github/downloads/RealViper8/steamtools/total" />
  <img src="https://img.shields.io/github/last-commit/RealViper8/steamtools" />
  <img src="https://img.shields.io/github/issues-raw/RealViper8/steamtools" />
  <img src="https://img.shields.io/github/repo-size/RealViper8/steamtools" />
</p>

<!-- ![Github Tag](https://img.shields.io/github/v/tag/RealViper8/steamtools)
![Github all Downloads](https://img.shields.io/github/downloads/RealViper8/steamtools/total)
![Github Last Commit](https://img.shields.io/github/last-commit/RealViper8/steamtools)
![Github Issues](https://img.shields.io/github/issues-raw/RealViper8/steamtools)
![GitHub repo size](https://img.shields.io/github/repo-size/RealViper8/steamtools) -->

**Steamtools** is a lightweight tool written in **Rust** that allows you to easily **view, install, and remove Lua Manifests**. The source code of the proxy dll: xinput1_4.dll is currently not available since I only ship the binary for it. May **change** in  the **future** !

> ⚠️ **Linux Support (WORK IN PROGRESS)**: Linux version (`.deb` package) is now available but still under development. Contributions welcome! See [Linux Support](#-linux-support-work-in-progress) section below.

---

## 🗓️ TODO

- [x] Add header image for Lua Manifests  
- [x] Add tool: Remove  
- [x] Add tool: Install  
- [x] Add tool: Uninstall  
- [X] Add tool: View  
- [X] Add Window: Settings
- [X] Search Bar
- [ ] Moving Mod, Plugins buttons down a bit for more space
- [ ] Improving Settings by adding (Start, Restart, Close Steam)

---

## ⚙️ Planned Side Features

- [X] Add MelonModLoader support
- [X] Add Lua Plugin support
  - [X] Basic Support (1 active running plugin)
  - [X] Code Editor (supports syntax highlighting)
  - [ ] Being able to run more then 1 script (on diffrent threads)
- [ ] Better Visuals

---

## Donating

Since this is a personal project and I invest my free time in making this you could support me by donating!
SOL: GYepZXvMAyQ8y54HuWx8QR3yBBg9EFEqn7dDJo93GbTM (SOL Network)

[![Donate](https://img.shields.io/badge/Donate-Ko--fi-orange)](https://ko-fi.com/realviper)

---

## 🐧 Linux Support (WORK IN PROGRESS)

⚠️ **Status**: Beta - Core functionality working, but still under active development

### What works:
- ✅ View installed Lua manifests
- ✅ Download manifests from multiple sources (ManifestHub, Ryuu, TwentyTwo, Sushi)
- ✅ Install manifests (.lua files)
- ✅ Automatic Steam path detection (including Debian package installations)
- ✅ Open in Steam button to launch installation directly
- ✅ Support for Debian package installations (`.steam/debian-installation/`)

### Installation (Linux):
```bash
wget https://github.com/RealViper8/steamtools/releases/download/vX.X.X/steamtools-0.6.3-linux.deb
sudo dpkg -i steamtools-0.6.3-linux.deb
steamtools
```

### Known Limitations:
- ⚠️ MelonLoader support not yet implemented for Linux
- ⚠️ Plugins feature is experimental
- ⚠️ Some UI elements may not render perfectly on all desktop environments

### Contributions:
Linux support is community-driven! Contributions welcome at [feat/linux-support](https://github.com/RealViper8/steamtools/tree/feat/linux-support) branch.

---

## Prerequisites

- VC_Runtime: [x64](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- Opengl Drivers:
  - [AMD](https://www.amd.com/en/support/download/drivers.html)
  - [NVIDIA](https://www.nvidia.com/de-de/drivers/)

**Language:** Rust 🦀
**Status:** In active development 🚀

### v0.2.1 Steamtools App

![Steamtools App](0.2.1.png)

### v0.5.1 GUI Rework

![GUI Rework](0.5.1.png)

![Alt](https://repobeats.axiom.co/api/embed/0c1c32f9f6d0794bc572fbb52c48eea466d46477.svg "Repobeats analytics image")
