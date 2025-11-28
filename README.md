# pm — Minimal Password Manager (Rust, CLI)

Локальный и безопасный менеджер паролей (Linux / macOS / Windows).

## 🔐 Возможности
- Локальное хранение (каждый сервис — отдельный `.enc`)
- Шифрование XChaCha20-Poly1305 (MK защищён Argon2id)
- Кэш Master Key на 5 минут
- OTP/TOTP (совместим с Google Auth, Aegis, GitHub и др.)
- Клипборд в GUI-терминале (`pm clip`, `pm otp clip`)
- Дерево записей (`pm ls`)

---

## 📦 Установка одной командой

### 🐧 Linux
```bash
curl -sSfL https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-linux.sh | bash
```

### 🍎 macOS
```bash
curl -sSfL https://raw.githubusercontent.com/Avdushin/pm/main/scripts/install-macos.sh | bash
```

### 🪟 Windows (PowerShell)
```powershell
irm https://github.com/Avdushin/pm/releases/latest/download/install.ps1 | iex
```

После установки проверь команду:

```bash
pm --help
```

## 🧰 Использование

Добавить хранилище:

```bash
pm init
```

Добавить запись (если пароль пустой → сгенерируется):

```bash
pm add work/github
```

Просмотреть:

```bash
pm show work/github
pm show work/github --password-only
pm show work/github --json
```

Список:

```bash
pm ls
pm ls work
```

Копировать пароль/логин:

```bash
pm clip work/github
pm clip work/github --field username
```

Добавить OTP (вставить base32 или otpauth://):

```bash
pm otp add work/github
```

Получить/копировать TOTP-код:

```bash
pm otp show work/github
pm otp clip work/github
```

Создать бэкап (`.zip` по умолчанию):

```bash
pm backup create
pm backup create my_backup
pm backup create my_backup.tar.gz
```

Заблокировать сессию (удалить кэш MK):

```bash
pm lock
pm backup lock
```

---

## 🤷 Формат записи

`.enc`:

```json
{ "version":1, "nonce":"<b64>", "ciphertext":"<b64>" }
```

