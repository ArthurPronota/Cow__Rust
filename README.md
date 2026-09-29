# `Cow` (Clone-on-Write) в Rust

## Что такое `Cow`

**`Cow<'a, B>`** — умный указатель из `std::borrow`, который может хранить **либо заимствованные** данные (`&T`), **либо владеемые** (`T`). **Идея:** пока изменения **не нужны** — держим **ссылку**; как только нужно **мутировать** — **клонируем** в `Owned`.

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),                    // ← заимствованные данные
    Owned(<B as ToOwned>::Owned),       // ← владеемые данные
}
```

## Два варианта

| Вариант | Что хранит | Аллокация |
|---|---|---|
| **`Cow::Borrowed(&B)`** | Ссылку | ❌ Нет |
| **`Cow::Owned(B::Owned)`** | Владеемое значение | ✅ Да |

## Разбор примера

```rust
use std::borrow::Cow;

fn my_normalize(s: &str) -> Cow<'_, str> {
    if s.contains("\n") {
        Cow::Owned(s.replace("\n", ""))   // ← есть \n → новое
    } else {
        Cow::Borrowed(s)                   // ← нет \n → ссылка
    }
}

fn main() {
    println!("{}", my_normalize("Hello, world!"));
    // Borrowed, БЕЗ аллокации → Hello, world!

    println!("{}", my_normalize("Hello\nworld!"));
    // Owned, с аллокацией → Helloworld!
}
```

### Что происходит

1. **`"Hello, world!"`** — **нет** `\n` → `Borrowed` (**без** аллокации).
2. **`"Hello\nworld!"`** — **есть** `\n` → `Owned` (**с** аллокацией).
3. **`Cow<str>`** **автоматически** **разыменовывается** в `&str` для `println!`.

## Когда `Cow` **реально** полезен

### 1. **Парсинг** с fallback

```rust
fn parse(s: &str) -> Cow<str> {
    if s.starts_with('"') && s.ends_with('"') {
        Cow::Borrowed(&s[1..s.len()-1])   // ← без кавычек, без аллокации
    } else {
        Cow::Owned(format!("\"{}\"", s))   // ← добавляем кавычки
    }
}
```

### 2. **Нормализация**

```rust
fn normalize(s: &str) -> Cow<str> {
    if s.contains('\t') {
        Cow::Owned(s.replace('\t', "    "))   // ← заменяем табы
    } else {
        Cow::Borrowed(s)                       // ← уже нормально
    }
}
```

### 3. `Path::to_string_lossy`

```rust
use std::path::Path;

let path = Path::new("/tmp/file.txt");
let s: Cow<str> = path.to_string_lossy();
```

- **`Borrowed`** — если путь **валидный** UTF-8.
- **`Owned`** — если есть **невалидные** байты (заменяются на `�`).

### 4. **Serde** с zero-copy

```rust
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize)]
struct Data<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,
}
```

- **`Borrowed`** — если в JSON **нет** escape-последовательностей.
- **`Owned`** — если есть `\n`, `\\`, `\"` — их нужно **раскодировать**.

### Почему escape-последовательности требуют `Owned`

```json
{
  "name": "Alice\nBob",
  "path": "C:\\Users",
  "quote": "He said \"Hello\""
}
```

**JSON-парсер** должен:

1. **Распознать** `\n`, `\\`, `\"`.
2. **Преобразовать** в **реальные** символы:
   - `\n` → перенос строки;
   - `\\` → обратный слеш;
   - `\"` → кавычка.
3. **Создать** новую строку (`Owned`).

**Если** escape-последовательностей **нет** → `Borrowed` (**zero-copy**).

## `Cow` и `Deref`

**`Cow<B>`** реализует **`Deref<Target = B>`**:

```rust
let s: Cow<str> = Cow::Borrowed("hello");

println!("{}", s.len());          // 5 — метод через Deref
println!("{}", s.to_uppercase()); // HELLO
```

## `to_mut` и `into_owned`

### `to_mut()` — гарантирует `Owned`

```rust
let mut s: Cow<str> = Cow::Borrowed("hello");

s.to_mut().push_str(" world");   // ← клонирует при необходимости

println!("{}", s);   // hello world
```

### `into_owned()` — извлекает `Owned`

```rust
let s: Cow<str> = Cow::Borrowed("hello");
let owned: String = s.into_owned();   // ← клонирует

println!("{}", owned);   // hello
```

## Схема `Cow`

```
Cow::Borrowed("hello"):
        ↓
     &str ──→ "hello" (в бинарнике / чужой буфер)

Cow::Owned(String::from("hello")):
        ↓
     String ──→ [heap: "hello"]
```

## Сводная таблица методов

| Метод | Что делает |
|---|---|
| **`Cow::Borrowed(&B)`** | Создаёт **заимствованный** |
| **`Cow::Owned(B::Owned)`** | Создаёт **владеемый** |
| **`to_mut()`** | Превращает в `Owned` (**клонирует** при необходимости) |
| **`into_owned()`** | Извлекает `Owned` (**клонирует** при необходимости) |
| **`as_ref()`** | Возвращает `&B` |
| **`Deref`** | Доступ к методам `B` |

## Когда `Cow` **не помогает**

| Ситуация | Проблема |
|---|---|
| **Всегда** нужна **мутация** | `Cow` **добавляет** ветвление |
| **Заведомо** нужен `String` | Берите `String` **напрямую** |
| **Горячий** путь с **мутацией** | `Cow` **замедляет** |

**Правило:** `Cow` — **только** если **иногда** нужна **мутация**, а **иногда** — **ссылка**.

## Сводная таблица применений

| Применение | Пример |
|---|---|
| **Условная** замена | `replace` при наличии `\n` |
| **Парсинг** с fallback | Убрать кавычки |
| **Нормализация** | Заменить табы |
| **`Path::to_string_lossy`** | UTF-8 lossy |
| **Serde** zero-copy | `Cow<'a, str>` |
| **Кэш** | `Cow<str>` |

## Сводная таблица

| Аспект | `Cow::Borrowed` | `Cow::Owned` |
|---|---|---|
| **Владение** | ❌ | ✅ |
| **Аллокация** | ❌ | ✅ |
| **Изменение** | Клонирует | Прямое |
| **`to_mut`** | Клонирует | Возвращает `&mut` |
| **Когда** | Данные **не менялись** | Данные **изменены** |

## Полный пример: JSON-парсинг (упрощённо)

```rust
use std::borrow::Cow;

fn unescape(s: &str) -> Cow<str> {
    if s.contains('\\') {
        // Есть escape-последовательности → Owned
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        Cow::Owned(result)
    } else {
        // Нет escape → Borrowed (zero-copy)
        Cow::Borrowed(s)
    }
}

fn main() {
    let s1 = "Hello, world!";
    let s2 = "Hello\\nworld!";

    match unescape(s1) {
        Cow::Borrowed(_) => println!("Borrowed: {}", unescape(s1)),
        Cow::Owned(_) => println!("Owned: {}", unescape(s1)),
    }
    // Borrowed: Hello, world!

    match unescape(s2) {
        Cow::Borrowed(_) => println!("Borrowed: {}", unescape(s2)),
        Cow::Owned(_) => println!("Owned: {}", unescape(s2)),
    }
    // Owned: Hello\nworld!
}
```

## Итог

- **`Cow<'a, B>`** — **Clone-on-Write** указатель.
- **Два варианта:**
  - **`Borrowed(&B)`** — **без** аллокации;
  - **`Owned(B::Owned)`** — **с** аллокацией.
- **`Deref`** — доступ к методам `B`.
- **`to_mut()`** — превращает в `Owned` (**клонирует** при необходимости).
- **`into_owned()`** — извлекает `Owned`.
- **Применения:**
  - **парсинг** с fallback;
  - **нормализация**;
  - **`Path::to_string_lossy`**;
  - **Serde** zero-copy;
  - **кэш**.
- **Serde:** escape-последовательности → `Owned`; без них → `Borrowed`.
- **Когда `Cow` не помогает:** всегда мутация, заведомо `String`, горячий путь.
- **В вашем примере:** `my_normalize` — `Borrowed` без `\n`, `Owned` с `\n`.
- **Правило:** `Cow` — когда **иногда** нужна **мутация**, а **иногда** — **ссылка**.
