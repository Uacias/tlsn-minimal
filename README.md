# iOS Mobile Playground - Integracja Rust z Xcode

## **1. Budowanie biblioteki Rust dla iOS**

Przejdź do katalogu projektu:
```sh
cd tlsn-example
```

Skonfiguruj i skompiluj bibliotekę Rust dla urządzeń fizycznych (`aarch64-apple-ios`) oraz emulatora (`x86_64-apple-ios`):
```sh
cargo build --target aarch64-apple-ios
cargo build --target x86_64-apple-ios
```

Połącz biblioteki w jedną uniwersalną:
```sh
lipo -create target/aarch64-apple-ios/debug/libtlsn_example.a target/x86_64-apple-ios/debug/libtlsn_example.a -output target/libtlsn.a
```

---

## **2. Konfiguracja projektu Xcode**

1. Otwórz projekt w Xcode:
```sh
open app/tlsn_example.xcodeproj
```
2. Przenieś skompilowany plik `target/libtlsn.a` do katalogu projektu.
3. Utwórz plik nagłówkowy `tlsn.h` i dodaj następującą treść:

```c
#ifndef tlsn_h
#define tlsn_h

#include <stdio.h>
const char *say_hello();

#endif /* tlsn_h */
```

4. Utwórz **Bridging Header** o nazwie `tlsn_example-Bridging-Header.h` i dodaj do niego:
```c
#include "tlsn.h"
```

---

## **3. Uruchomienie weryfikatora Rust**

Przejdź do katalogu `verifier` i uruchom weryfikator:
```sh
cd verifier
cargo run
```

---

## **4. Uruchomienie serwera TLSNotary**

Sklonuj repozytorium `tlsn` i uruchom serwer:
```sh
git clone https://github.com/tlsnotary/tlsn.git
cargo run PORT=3000 cargo run --bin tlsn-server-fixture
```

---

## **5. Uruchomienie aplikacji Swift**
Teraz uruchom aplikację w Xcode, aby przetestować integrację Rust z Swift!

---

### **📌 Podsumowanie**
✅ Skonfigurowano Rust do kompilacji na iOS.
✅ Połączono `lipo`, aby utworzyć uniwersalną bibliotekę.
✅ Zaimportowano bibliotekę do Xcode.
✅ Uruchomiono weryfikator i serwer TLSNotary.
✅ Uruchomiono aplikację.

Gotowe do testowania! 🚀

---
## **6. Podczas tworzenia nowej biblioteki dodaj w Cargo.toml**
```
[lib]
crate-type = ["lib", "staticlib"]
```