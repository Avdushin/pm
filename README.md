# pm — Password Manager

**pm** — кроссплатформенный, минималистичный менеджер паролей для локального и безопасного хранения.  
Создавался как удобная и защищённая альтернатива `pass` с нормальной поддержкой бэкапов, OTP/TOTP и генерацией паролей.

---

## 🔐 Основные принципы
- Пароли хранятся **локально**, каждый сервис — отдельный зашифрованный файл
- Шифрование: **XChaCha20-Poly1305**
- Master Key (MK) сам хранится **в зашифрованном виде** в `config.json`, расшифровывается только через KEK
- KEK деривируется через **Argon2id** (ресурсоёмкая защита от перебора)
- MK **кэшируется на 5 минут**, как `sudo` или `gpg-agent`, чтобы не вводить пароль постоянно
- Поддерживает OTP/TOTP (совместимо с Google Auth, Aegis, GitHub и др.)
- Есть копирование в clipboard (X11, Wayland, macOS, Windows, Linux GUI терминалы)
- Поддерживает группировку по “папкам”: `work/github`, `personal/email`, `crypto/binance` и т.п.
- Команда `pm ls` выводит дерево записей как `pass ls`

---

## 📦 Установка одной командой

### Linux (через install-скрипт, нужен Rust/cargo)

Собрать из исходников и поставить в `~/.local/bin/pm`:

```bash
curl -sSfL https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-linux.sh | bash
````

По умолчанию скрипт ставит бинарник в `~/.local/bin`. Убедись, что он есть в `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

---

### macOS (через install-скрипт, нужен Rust/cargo)

```bash
curl -sSfL https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-macos.sh | bash
```

По умолчанию скрипт ставит в `/usr/local/bin/pm` (может потребоваться `sudo`).

---

### Linux (готовый бинарник из релиза)

```bash
mkdir -p "$HOME/.local/bin"
curl -sSfL https://github.com/Avdushin/pm/releases/latest/download/pm-linux-amd64 -o "$HOME/.local/bin/pm" && \
chmod 700 "$HOME/.local/bin/pm" && \
echo "✅ pm установлен в $HOME/.local/bin/pm"
```

---

### macOS (готовый бинарник из релиза)

```bash
sudo curl -sSfL https://github.com/Avdushin/pm/releases/latest/download/pm-macos-amd64 -o /usr/local/bin/pm && \
sudo chmod 755 /usr/local/bin/pm && \
echo "✅ pm установлен в /usr/local/bin/pm"
```

> Убедись, что каталог установки находится в `PATH`, например:
>
> ```bash
> export PATH="$HOME/.local/bin:$PATH"
> ```

---

### Windows (PowerShell, через install-скрипт)

Сборка и установка в `%USERPROFILE%\.cargo\bin\pm.exe`:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
irm https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-windows.ps1 | iex
```

---

### Windows (готовый бинарник из релиза)

```powershell
$dest = "$env:USERPROFILE\.cargo\bin"
New-Item -ItemType Directory -Path $dest -Force | Out-Null
Invoke-WebRequest -UseBasicParsing https://github.com/Avdushin/pm/releases/latest/download/pm-windows-amd64.exe -OutFile "$dest\pm.exe"
Write-Host "✅ pm установлен в $dest\pm.exe"
```

---

## ⚙️ Ручная установка и сборка из исходников

```bash
git clone https://github.com/Avdushin/pm
cd pm
cargo build --release

# Установить в PATH (Linux)
sudo cp target/release/pm /usr/local/bin/pm
```

**Если нужен clipboard на Arch/X11:**

```bash
sudo pacman -S xclip
# или для Wayland:
sudo pacman -S wl-clipboard
```

---

## 🧰 Использование

### 1. Инициализация хранилища

```bash
pm init
```

Будет диалог:

```text
New master password: *********
Confirm master password: *********
Initialized store at /home/<user>/.local/share/pm-store
```

Создаётся структура:

```text
~/.local/share/pm-store/
├── config.json
└── store/       # все записи *.enc
```

---

### 2. Добавление пароля

```bash
pm add work/github
```

Пример диалога:

```text
Master password: *********  # (пропускается, если MK в кэше)
Username (optional): user@mail.com
Password (leave empty to generate):  # ← оставим пустым для генерации
URL (optional): https://github.com
Notes: GitHub account
Saved entry work/github
```

Если пароль пустой → сгенерируется автоматически.

---

### 3. Просмотр пароля

```bash
pm show work/github
```

Вывод:

```text
Title:    work/github
Username: user@mail.com
Password: <password>
URL:      https://github.com
Notes:    GitHub account
Created:  2025-11-28T17-13-31Z
Updated:  2025-11-28T17-13-31Z
OTP:      not set        # или "configured", если уже привязан
```

Доп. варианты:

```bash
pm show work/github --password-only
pm show work/github --json
```

---

### 4. Просмотр списка записей

```bash
pm ls
pm ls work   # фильтр по "каталогу"
```

Пример:

```text
work/github
work/gitlab
work/jira
personal/email
crypto/binance
```

---

### 5. Копирование в clipboard

```bash
pm clip work/github
pm clip work/github --field username
```

Проверить содержимое на Arch/X11:

```bash
xclip -o
```

> Команда `pm clip` работает только в GUI-сессии (`$XDG_SESSION_TYPE != tty`).
> В `tty` (голой консоли) глобального клипборда не существует.

---

### 6. Работа с OTP/TOTP (двухфакторная аутентификация)

#### 6.1. Привязать OTP к записи

```bash
pm otp add work/github
```

Можно вставить **либо просто base32-секрет**, либо **полную ссылку otpauth://**, как её даёт сервис:

```text
otpauth://totp/GitHub%2FUser?period=30&digits=6&algorithm=SHA1&secret=XA5LJ***&issuer=GitHub
```

(функция сама распарсит параметры: `secret`, `digits`, `period`, `algorithm`)

Увидишь:

```text
OTP configured for work/github
```

---

#### 6.2. Показать текущий TOTP-код

```bash
pm otp show work/github
```

Выводит 6-значный актуальный код:

```text
730056
```

(меняется каждые `period` секунд, обычно 30)

---

#### 6.3. Скопировать код OTP в clipboard

```bash
pm otp clip work/github
```

```text
OTP code copied to clipboard.
```

---

### 7. Создать бэкап (по умолчанию ZIP)

```bash
pm backup create
# создаст: backup_<timestamp>.zip
```

Можно указать своё имя, опционально с расширением:

```bash
pm backup create my_backup
pm backup create my_backup.zip    # форс-zip
pm backup create my_backup.tar.gz # TAR.GZ вместо ZIP
```

---

### 8. Заблокировать сессию (удалить кэш MK)

```bash
pm backup lock
# или
pm lock
```

---

## 📦 Формат хранимых записей

Каждый файл `store/<path>.enc` выглядит так:

```json
{
  "version": 1,
  "nonce": "<base64>",
  "ciphertext": "<base64>"
}
```

Внутри `ciphertext` лежит сериализованный `Entry`.

---

## 💾 Бэкап и восстановление

На данный момент есть только **создание** бэкапа.
Архив содержит **уже зашифрованные файлы**, поэтому он безопасен в хранении и переносе.

> В будущем можно добавить: `pm backup restore FILE` для распаковки и импорта.

---

## 🧠 Trade-offs безопасности

`pm` использует временный кэш `pm-session.json`:

* Путь на Linux: `$XDG_RUNTIME_DIR/pm-session.json` (обычно `/run/user/1000/pm-session.json`)
* Права: `600`
* Хранит: `master_key_base64`, `cached_at` и `ttl`

Кэш живёт только 5 минут или до блокировки.

---

## 🛠 Планы

* `pm otp remove`
* `pm backup restore`
* импорт/экспорт записей
* GUI поверх core (tauri/iced/egui/eframe)

