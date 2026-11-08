# Руководство по установке Lacie

В этом документе описаны способы установки и настройки пакетного менеджера **Lacie** для Linux, macOS, Windows 11, FreeBSD и OpenBSD.

---

## Содержание

- [Системные требования](#системные-требования)
- [Обзор способов установки](#обзор-способов-установки)
- [Linux](#linux)
  - [Способ 1: Интерактивный установщик Ruby (Рекомендуется)](#linux---способ-1-интерактивный-установщик-ruby-рекомендуется)
  - [Способ 2: Nix](#linux---способ-2-nix)
  - [Способ 3: Прямая загрузка бинарного файла](#linux---способ-3-прямая-загрузка-бинарного-файла)
- [macOS](#macos)
  - [Способ 1: Интерактивный установщик Ruby (Рекомендуется)](#macos---способ-1-интерактивный-установщик-ruby-рекомендуется)
  - [Способ 2: Nix](#macos---способ-2-nix)
  - [Способ 3: Прямая загрузка бинарного файла](#macos---способ-3-прямая-загрузка-бинарного-файла)
- [Windows 11](#windows-11)
  - [Способ 1: Интерактивный установщик Ruby](#windows-11---способ-1-интерактивный-установщик-ruby)
  - [Способ 2: Прямая загрузка бинарного файла](#windows-11---способ-2-прямая-загрузка-бинарного-файла)
- [FreeBSD](#freebsd)
  - [Способ 1: Интерактивный установщик Ruby](#freebsd---способ-1-интерактивный-установщик-ruby)
  - [Способ 2: Прямая загрузка бинарного файла](#freebsd---способ-2-прямая-загрузка-бинарного-файла)
- [OpenBSD](#openbsd)
  - [Способ 1: Интерактивный установщик Ruby](#openbsd---способ-1-интерактивный-установщик-ruby)
  - [Способ 2: Прямая загрузка бинарного файла](#openbsd---способ-2-прямая-загрузка-бинарного-файла)
- [Настройка после установки](#настройка-после-установки)
  - [Настройка переменной PATH](#настройка-переменной-path)
  - [Инициализация окружения (`init-lacie`)](#инициализация-окружения-init-lacie)
  - [Проверка работоспособности (`doctor`)](#проверка-работоспособности-doctor)
- [Обновление Lacie](#обновление-lacie)
- [Удаление Lacie](#удаление-lacie)

---

## Системные требования

- **Сеть:** Доступ к интернету для загрузки релизов с GitHub (`github.com/shareui/lacie`).
- **Ruby:** Для работы интерактивного скрипта установки (`install.rb`) рекомендуется Ruby 3.0+ (присутствует по умолчанию или устанавливается через штатный менеджер пакетов большинства Unix-систем).
- **Права доступа:** Права суперпользователя (root/administrator) **не требуются**. Lacie работает целиком в пользовательском пространстве (`~/.local/bin` или выбранная директория, сохраняя установленные пакеты в `$HOME/lacie` или `%LOCALAPPDATA%\lacie`).

---

## Обзор способов установки

| Платформа | Установщик Ruby (`install.rb`) | Nix (`flake.nix`) | Прямая загрузка бинарника |
| :--- | :---: | :---: | :---: |
| **Linux (GNU / musl, x86_64 / aarch64)** | Поддерживается (Рекомендуется) | Поддерживается | Поддерживается |
| **macOS (Intel / Apple Silicon)** | Поддерживается (Рекомендуется) | Поддерживается | Поддерживается |
| **Windows 11 (x86_64)** | Поддерживается | Н/Д | Поддерживается |
| **FreeBSD (x86_64)** | Поддерживается | Н/Д | Поддерживается |
| **OpenBSD (x86_64)** | Поддерживается | Н/Д | Поддерживается |

---

## Linux

### Linux - Способ 1: Интерактивный установщик Ruby (Рекомендуется)

Скачайте скрипт и запустите установщик локально:

```bash
curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
ruby install.rb
```

**Что делает скрипт установки:**
1. Автоматически определяет операционную систему (`Linux`), C-библиотеку (`GNU` или `musl`) и процессорную архитектуру (`x86_64` или `aarch64`).
2. Запрашивает список доступных релизов из репозитория `shareui/lacie` и предлагает выбрать версию.
3. Сканирует системную переменную `$PATH` на наличие каталогов, доступных пользователю для записи (например, `~/.local/bin` или `~/bin`), и позволяет выбрать путь назначения.
4. Загружает соответствующий бинарный файл напрямую с GitHub Releases с индикатором прогресса.
5. Выставляет права на исполнение (`chmod 0755`) и запускает инициализацию `lacie init-lacie`.

### Linux - Способ 2: Nix

Если на машине используется Nix со включенной поддержкой Flakes:

**Запуск без явной установки:**
```bash
nix run github:shareui/lacie -- --help
```

**Установка в профиль пользователя:**
```bash
nix profile install github:shareui/lacie
```

**Подключение через `flake.nix` (NixOS или Home Manager):**
```nix
{
  inputs = {
    lacie.url = "github:shareui/lacie";
  };

  outputs = { self, nixpkgs, lacie }: {
    # Добавьте lacie.packages.${system}.default в environment.systemPackages или home.packages
  };
}
```

### Linux - Способ 3: Прямая загрузка бинарного файла

Скачайте готовый бинарник под вашу архитектуру со страницы [GitHub Releases](https://github.com/shareui/lacie/releases):

- `lacie-linux-gnu-x86_64` (glibc x86_64)
- `lacie-linux-gnu-aarch64` (glibc ARM64)
- `lacie-linux-musl-x86_64` (musl x86_64, Alpine Linux / Void Linux)
- `lacie-linux-musl-aarch64` (musl ARM64)

```bash
# Пример для Linux x86_64 GNU
curl -fsSL -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-linux-gnu-x86_64
chmod +x ~/.local/bin/lacie
lacie init-lacie
```

---

## macOS

### macOS - Способ 1: Интерактивный установщик Ruby (Рекомендуется)

В macOS интерпретатор Ruby доступен в системе или устанавливается через Homebrew (`brew install ruby`):

```bash
curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
ruby install.rb
```

Скрипт определит процессор (Apple Silicon `aarch64` на чипах M1/M2/M3/M4 или Intel `x86_64`), предложит выбрать директорию (например, `~/.local/bin`) и завершит установку.

### macOS - Способ 2: Nix

На macOS с установленным Nix:

```bash
nix profile install github:shareui/lacie
```

Либо запуск:

```bash
nix run github:shareui/lacie -- --help
```

### macOS - Способ 3: Прямая загрузка бинарного файла

Скачайте бинарник со страницы [GitHub Releases](https://github.com/shareui/lacie/releases):
- `lacie-macos-aarch64` (Apple Silicon)
- `lacie-macos-x86_64` (Intel)

```bash
# Пример для Apple Silicon
mkdir -p ~/.local/bin
curl -fsSL -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-macos-aarch64
chmod +x ~/.local/bin/lacie
lacie init-lacie
```

---

## Windows 11

### Windows 11 - Способ 1: Интерактивный установщик Ruby

Если установлен Ruby (через RubyInstaller или команду `winget install RubyInstallerTeam.Ruby.3.3`):

1. Откройте PowerShell или Windows Terminal.
2. Скачайте и запустите установщик:
   ```powershell
   Invoke-WebRequest -Uri "https://raw.githubusercontent.com/shareui/lacie/main/install.rb" -OutFile "install.rb"
   ruby install.rb
   ```
3. Следуйте интерактивным подсказкам для установки в каталог пользовательских программ (например, `%LOCALAPPDATA%\Programs\bin`).

### Windows 11 - Способ 2: Прямая загрузка бинарного файла

1. Загрузите файл `lacie-windows-msvc-x86_64.exe` со страницы [GitHub Releases](https://github.com/shareui/lacie/releases).
2. Переименуйте его в `lacie.exe`.
3. Поместите файл в каталог, находящийся в переменной окружения `PATH` (например, `%LOCALAPPDATA%\Microsoft\WindowsApps` или пользовательский bin-каталог).
4. Выполните:
   ```cmd
   lacie init-lacie
   ```

---

## FreeBSD

### FreeBSD - Способ 1: Интерактивный установщик Ruby

Убедитесь, что Ruby установлен (`pkg install ruby`):

```sh
fetch https://raw.githubusercontent.com/shareui/lacie/main/install.rb
ruby install.rb
```

Либо с помощью `curl`:
```sh
curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
ruby install.rb
```

### FreeBSD - Способ 2: Прямая загрузка бинарного файла

Загрузите `lacie-freebsd-x86_64` со страницы [GitHub Releases](https://github.com/shareui/lacie/releases):

```sh
mkdir -p ~/.local/bin
fetch -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-freebsd-x86_64
chmod 0755 ~/.local/bin/lacie
lacie init-lacie
```

---

## OpenBSD

### OpenBSD - Способ 1: Интерактивный установщик Ruby

Убедитесь, что Ruby и curl установлены (`pkg_add ruby curl`):

```sh
curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
ruby install.rb
```

### OpenBSD - Способ 2: Прямая загрузка бинарного файла

Загрузите `lacie-openbsd-x86_64` со страницы [GitHub Releases](https://github.com/shareui/lacie/releases):

```sh
mkdir -p ~/.local/bin
ftp -o ~/.local/bin/lacie https://github.com/shareui/lacie/releases/latest/download/lacie-openbsd-x86_64
chmod 0755 ~/.local/bin/lacie
lacie init-lacie
```

---

## Настройка после установки

### Настройка переменной PATH

Lacie размещает исполняемые ссылки на активные пакеты в `$HOME/lacie/active` (в Unix) или `%LOCALAPPDATA%\lacie\active` (в Windows).

Чтобы установленные через `lacie add` утилиты были доступны в командной строке:

**Bash / Zsh:**
Добавьте в `~/.bashrc` или `~/.zshrc`:
```bash
export PATH="$HOME/lacie/active:$PATH"
```

**Fish:**
Добавьте в `~/.config/fish/config.fish`:
```fish
fish_add_path -g $HOME/lacie/active
```

**PowerShell (Windows):**
```powershell
[Environment]::SetEnvironmentVariable("PATH", "$env:PATH;$env:LOCALAPPDATA\lacie\active", "User")
```

### Инициализация окружения (`init-lacie`)

Интерактивный установщик запускает эту команду автоматически. При ручной установке выполните:

```bash
lacie init-lacie
```

Команда сформирует необходимую файловую структуру:
- Unix: `$HOME/lacie/` (`config.toml`, `packages.toml`, `sources.toml`, `active/`, `bin/`, `scripts/`, `plugins/`, `var/`)
- Windows: `%LOCALAPPDATA%\lacie\`

### Проверка работоспособности (`doctor`)

Проверьте корректность структуры каталогов, конфигурационных файлов и путей:

```bash
lacie doctor
```

Если обнаружены проблемы, запустите автоматическое исправление:

```bash
lacie doctor --fix
```

---

## Обновление Lacie

Для обновления Lacie до свежей версии:

1. **Через установщик Ruby:**
   Запустите `install.rb` и выберите пункт `2` (`Update / Reinstall Lacie`). Скрипт найдет существующий исполняемый файл `lacie` в `$PATH` и атомарно заменит его:
   ```bash
   curl -fsSL -O https://raw.githubusercontent.com/shareui/lacie/main/install.rb
   ruby install.rb
   ```

2. **Через Nix:**
   ```bash
   nix profile upgrade github:shareui/lacie
   ```

3. **Вручную:**
   Замените бинарный файл свежим со страницы GitHub Releases.

---

## Удаление Lacie

Для полного и чистого удаления Lacie предусмотрена встроенная команда:

```bash
lacie suicide
```

Команда запросит подтверждение, корректно удалит все установленные пакеты, симлинки, кэш и каталоги данных (`$HOME/lacie`). После этого удалите сам бинарник `lacie` из вашего bin-каталога.
