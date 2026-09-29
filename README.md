# InitProject

Inicializador escrito en **Rust** para aplicaciones de consola en **C, C++, C#,
Rust y Python 3.14**. Genera los archivos del proyecto y compila o prepara su
entorno antes de indicar que la creación terminó correctamente.

## Compilar el inicializador

Instala una versión estable reciente de Rust con Cargo y ejecuta:

```sh
cargo build --locked --release
./target/release/InitProject --help
```

En Windows el ejecutable es `target/release/InitProject.exe`. Las plantillas de
C/C++ están orientadas a Linux/POSIX; las nuevas plantillas usan sus herramientas
multiplataforma. La comprobación automatizada se ejecuta en Linux.

## Requisitos por lenguaje

| Lenguaje | Herramientas necesarias | Resultado |
| --- | --- | --- |
| C | GCC, Make y `nob.h`; Python 3 para ejecutar los tests generados | `src/`, `include/`, `test/`, `bin/`, Makefile y `nob.c` |
| C++ | Compilador C++20, CMake >= 3.25, herramienta de build y `nob.h` | `src/`, `include/`, `test/`, `build/`, CMakeLists.txt y `nob.c` |
| C# | SDK de .NET con la plantilla `console` | `Program.cs`, `<nombre>.csproj`, `bin/` y `obj/` |
| Rust | Rust y Cargo | Cargo.toml, Cargo.lock, `src/main.rs` y `target/` |
| Python | uv reciente con soporte para Python 3.14 | pyproject.toml, uv.lock, `.python-version`, `main.py` y `.venv/` |

Git es necesario salvo que se indique `--no-git`. Los proyectos incluyen README
y `.gitignore`; los archivos de bloqueo se conservan para versionarlos.

La plantilla de C# usa `dotnet new console` y el framework predeterminado del
SDK instalado, seguido de `dotnet build`. La plantilla Rust es un binario de
edición 2021 que se compila con `cargo build` y tiene su propio workspace para
no modificar uno superior. Python usa `uv init` y `uv sync`: exige
`>=3.14,<3.15` y fija `3.14` en `.python-version`. uv puede descargar Python si
falta; las descargas y restauraciones pueden necesitar acceso a Internet.

## Uso

Desde el directorio donde quieras crear el proyecto, ejecuta el binario instalado
o sustituye `InitProject` por su ruta absoluta:

```sh
InitProject demo_c C
InitProject demo_cpp CPP
InitProject demo_cs 'C#'
InitProject demo_rust Rust
InitProject demo_python Python3.14
InitProject demo_sin_git PY --no-git
```

Sin argumentos pregunta nombre y lenguaje; si solo das el nombre, pregunta el
lenguaje. Los alias no distinguen mayúsculas:

| Lenguaje | Alias |
| --- | --- |
| C | `C` |
| C++ | `C++`, `CPP` |
| C# | `C#`, `CS`, `CSHARP` |
| Rust | `RUST`, `RS` |
| Python 3.14 | `PYTHON`, `PY`, `PYTHON3`, `PYTHON3.14` |

El nombre debe comenzar por una letra ASCII y contener solo letras ASCII,
números o `_`. Se crea una carpeta con ese nombre en el directorio actual; no
se aceptan rutas. `main` está reservado para C/C++ porque colisionaría con su
archivo de entrada. Se acepta un destino inexistente o un directorio vacío;
los destinos no vacíos se rechazan sin sobrescribir sus archivos.

Por defecto se inicializa Git en la rama `main`, sin crear ningún commit.
`--no-git` evita crear el repositorio, también con uv. Las herramientas se
comprueban antes de crear la carpeta; si una compilación o descarga posterior
falla, se devuelve un error y se conservan los archivos generados para poder
diagnosticarlo.

## Localización de nob.h (solo C/C++)

Se busca antes de entrar al nuevo proyecto, en este orden:

1. La ruta de archivo indicada por `NOB_PATH` (absoluta o relativa al directorio inicial).
2. Junto al ejecutable InitProject.
3. `/usr/local/share/InitProject/nob.h`.
4. El directorio desde el que se invoca InitProject.

Por ejemplo, desde la raíz de este repositorio:

```sh
NOB_PATH="$PWD/nob.h" ./target/release/InitProject demo_c C
```

C#, Rust y Python no necesitan `nob.h`. En C, los objetos se compilan con
`-fPIC` para poder enlazar la biblioteca compartida de los tests.

## Ejecutar el proyecto generado

Después de entrar en su carpeta:

| Lenguaje | Comando |
| --- | --- |
| C | `make run` |
| C++ | `./build/<nombre>` (generador de configuración única) |
| C# | `dotnet run` |
| Rust | `cargo run` |
| Python 3.14 | `uv run python main.py` |

Para agregar dependencias Python: `uv add nombre_del_paquete`.

## Comprobaciones de integración

Con todas las herramientas de la tabla instaladas:

```sh
cargo build --locked
python3 tests/smoke.py
```

Las pruebas crean proyectos temporales de los cinco lenguajes, ejecutan los
programas resultantes y comprueban `--no-git`, Python 3.14, el descubrimiento de
`nob.h`, la protección de destinos existentes y el aislamiento de workspaces.
El workflow de GitHub Actions instala las herramientas y ejecuta esta misma
comprobación. C# se prueba con .NET 10.
