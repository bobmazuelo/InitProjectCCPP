use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

// ─── Colores ANSI ─────────────────────────────────────────────────────────────
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

macro_rules! ok {
    ($($arg:tt)*) => {
        println!("{}✔  {}{}", GREEN, format!($($arg)*), RESET)
    };
}
macro_rules! info {
    ($($arg:tt)*) => {
        println!("{}➜  {}{}", CYAN, format!($($arg)*), RESET)
    };
}
macro_rules! err {
    ($($arg:tt)*) => {
        eprintln!("{}✖  ERROR: {}{}", RED, format!($($arg)*), RESET)
    };
}

// ─── Búsqueda de nob.h ───────────────────────────────────────────────

fn find_header(env_var: &str, filename: &str) -> Option<PathBuf> {
    // 1. Variable de entorno
    if let Ok(val) = std::env::var(env_var) {
        let p = PathBuf::from(val);
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. Junto al ejecutable
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap_or(Path::new(".")).join(filename);
        if p.is_file() {
            return Some(p);
        }
    }

    // 3. /usr/local/share/InitProject/
    let system = PathBuf::from(format!("/usr/local/share/InitProject/{}", filename));
    if system.is_file() {
        return Some(system);
    }

    // 4. Directorio de trabajo actual
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .join(filename);
    if cwd.is_file() {
        return Some(cwd);
    }

    None
}

fn copy_header(src: &Path, dest: &str) -> Result<(), String> {
    fs::copy(src, dest)
        .map(|_| ())
        .map_err(|e| format!("No se pudo copiar {:?} → {}: {}", src, dest, e))
}

// ─── Modo interactivo ─────────────────────────────────────────────────────────

fn prompt(msg: &str) -> String {
    print!("{}{}{} ", BOLD, msg, RESET);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

// ─── Ejecutar comandos ────────────────────────────────────────────────────────

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = process::Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("No se pudo ejecutar '{} {}': {}", cmd, args.join(" "), e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
                "'{} {}' salió con código {}",
                cmd,
                args.join(" "),
                status.code().unwrap_or(-1)
        ))
    }
}

// ─── Proyecto C ───────────────────────────────────────────────────────────────

fn create_c_project(name: &str, git: bool, nob_src: &Path) -> Result<(), String> {
    let upper = name.to_uppercase();

    // Directorios
    for dir in &["src", "include", "test", "bin"] {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
    }

    // nob.h
    copy_header(nob_src, "nob.h")?;
    ok!("nob.h copiado desde {:?}", nob_src);

    // Makefile
    write_file(
        "Makefile",
        &format!(
            r#"SRDIR   = src/
TESTDIR = test/
SRC     = $(wildcard $(SRDIR)*.c)
NAME    = bin/{name}
OBJS    = $(SRC:.c=.o)
CC      = gcc
CFLAGS  = -Wall -Wextra -Werror -fPIC -Iinclude
TFLAGS  = -shared -fPIC -Iinclude
RM      = rm -rf

all: $(NAME)

$(SRDIR)%.o: $(SRDIR)%.c
{0}$(CC) $(CFLAGS) -c $< -o $@

$(NAME): $(OBJS)
{0}$(CC) $(CFLAGS) -o $(NAME) $(OBJS)

lib: $(OBJS)
{0}$(CC) $(TFLAGS) -o $(TESTDIR)libtest.so $(OBJS)
{0}chmod +x $(TESTDIR)test.py

clean:
{0}$(RM) $(OBJS)

fclean: clean
{0}$(RM) $(NAME) $(TESTDIR)libtest.so

run:
{0}./$(NAME)

re: fclean all

.PHONY: all clean fclean re run lib
"#, "\t"
),
        )?;

    // src/main.c
    write_file(
        "src/main.c",
        &format!(
            r#"#include <{name}.h>

int{0}main(int argc, char **argv)
{{
{0}int nb = 42;

{0}if (argc && argv)
{0}{0}printf("Hello, World!\n%d * %d == %d\n", nb, nb, square(nb));

{0}return (0);
}}
"#, "\t"
        ),
    )?;

// src/<name>.c
write_file(
    &format!("src/{name}.c"),
    &format!(
        r#"#include <{name}.h>

int{0}square(int num)
{{
{0}return (num * num);
}}
"#, "\t"
    ),
)?;

// include/<name>.h
write_file(
    &format!("include/{name}.h"),
    &format!(
        r#"#ifndef {upper}_H
#define {upper}_H

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define errf(fmt, args...) do {{ fprintf(stderr, "ERROR @ %s(): ", __func__); fprintf(stderr, fmt, ##args); }} while (0)
#define ERROR_EXIT(...)    do {{ fprintf(stderr, __VA_ARGS__); exit(1); }} while (0)
#define ERROR_RETURN(R, ...) do {{ fprintf(stderr, __VA_ARGS__); return R; }} while (0)

int{0}square(int);

#endif
"#, "\t"
    ),
)?;

// nob.c
write_file(
    "nob.c",
    &format!(
        r#"#define NOB_IMPLEMENTATION
#include "nob.h"
#include <string.h>
#include <stdio.h>
#include <unistd.h>

int{0}main(int argc, char **argv)
{{
{0}NOB_GO_REBUILD_URSELF(argc, argv);
{0}nob_shift_args(&argc, &argv);

{0}Nob_Cmd cmd = {{0}};
{0}int clean_executed = 0;

{0}nob_cmd_append(&cmd, "make", NULL);
{0}if (!nob_cmd_run_sync(cmd)) return (1);

{0}if (argc > 0)
{0}{{
{0}{0}const char *subcmd = nob_shift_args(&argc, &argv);

{0}{0}if (strcmp(subcmd, "test") == 0)
{0}{0}{{
{0}{0}{0}cmd.count = 0;
{0}{0}{0}nob_cmd_append(&cmd, "make", "lib");
{0}{0}{0}nob_da_append_many(&cmd, argv, argc);
{0}{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);

{0}{0}cmd.count = 0;
{0}{0}nob_cmd_append(&cmd, "make", "clean");
{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);
{0}{0}clean_executed = 1;

{0}{0}if (chdir("./test") != 0) {{ perror("chdir"); return (1); }}

{0}{0}cmd.count = 0;
{0}{0}nob_cmd_append(&cmd, "./test.py", NULL);
{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);

{0}{0}if (chdir("../") != 0) {{ perror("chdir"); return (1); }}
{0}{0}}}
{0}{0}else if (strcmp(subcmd, "run") == 0)
{0}{0}{{
{0}{0}{0}cmd.count = 0;
{0}{0}{0}nob_cmd_append(&cmd, "./bin/{name}", NULL);
{0}{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);
{0}{0}}}
{0}{0}else
{0}{0}{{
{0}{0}{0}nob_log(NOB_ERROR, "Subcomando desconocido: %s", subcmd);
{0}{0}{0}return (1);
{0}{0}}}
{0}}}

{0}if (!clean_executed)
{0}{{
{0}{0}cmd.count = 0;
{0}{0}nob_cmd_append(&cmd, "make", "clean");
{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);
{0}}}

{0}return (0);
}}
"#, "\t"
),
    )?;

// test/test.py
write_file(
    "test/test.py",
    &format!(
        r#"#!/usr/bin/python3

import ctypes
import unittest

mylib = ctypes.CDLL('./libtest.so')
mylib.square.restype = ctypes.c_int
mylib.square.argtypes = [ctypes.c_int]

class SquareTest(unittest.TestCase):
{0}def test_square_positive(self):
{0}{0}self.assertEqual(mylib.square(2), 4)
{0}{0}self.assertEqual(mylib.square(0), 0)
{0}{0}self.assertEqual(mylib.square(42), 1764)

{0}def test_square_negative(self):
{0}{0}self.assertEqual(mylib.square(-3), 9)

if __name__ == '__main__':
{0}unittest.main()
    "#, "\t"
    ),
    )?;

write_readme(name, "C")?;
write_gitignore_c()?;

run("make", &["all"])?;
run("make", &["lib"])?;
run("make", &["clean"])?;
run("make", &["run"])?;

if git {
    init_git()?;
}

ok!("Proyecto C '{name}' creado con éxito.");
Ok(())
    }

// ─── Proyecto C++ ─────────────────────────────────────────────────────────────

fn create_cpp_project(name: &str, git: bool, nob_src: &Path) -> Result<(), String> {
    let upper = name.to_uppercase();

    for dir in &["src", "include", "test", "build"] {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
    }

    // nob.h
    copy_header(nob_src, "nob.h")?;
    ok!("nob.h copiado desde {:?}", nob_src);

    // CMakeLists.txt
    write_file(
        "CMakeLists.txt",
        &format!(
            r#"cmake_minimum_required(VERSION 3.25)

project({name})

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(SRCS
    src/main.cpp
    src/{name}.cpp
)

include_directories(${{CMAKE_SOURCE_DIR}}/include)

add_executable({name} ${{SRCS}})
"#
        ),
    )?;

    // src/main.cpp
    write_file(
        "src/main.cpp",
        &format!(
            r#"#include <{name}.h>

int main(int argc, char **argv)
{{
{0}if (argc && argv)
{0}{0}std::cout << "Hello from {name}!" << std::endl;
{0}return (0);
}}
"#, "\t"
        ),
    )?;

// src/<name>.cpp
write_file(
    &format!("src/{name}.cpp"),
    &format!(
        r#"#include <{name}.h>

// Implementa aquí las funciones de {name}
        "#
    ),
)?;

// include/<name>.h
write_file(
    &format!("include/{name}.h"),
    &format!(
        r#"#ifndef {upper}_H
#define {upper}_H

#include <cstdio>
#include <cstdlib>
#include <unistd.h>
#include <iostream>
#include <string>

#define errf(fmt, args...) do {{ printf("ERROR @ %s(): ", __func__); printf(fmt, ##args); }} while (0)
#define ERROR_EXIT(...)    do {{ fprintf(stderr, __VA_ARGS__); exit(1); }} while (0)
#define ERROR_RETURN(R, ...) do {{ fprintf(stderr, __VA_ARGS__); return R; }} while (0)

#endif
        "#
    ),
)?;

// nob.c 
write_file(
    "nob.c",
    &format!(
        r#"#define NOB_IMPLEMENTATION
#include "nob.h"
#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>

int{0}main(int argc, char **argv)
{{
{0}NOB_GO_REBUILD_URSELF(argc, argv);
{0}nob_shift_args(&argc, &argv);

{0}Nob_Cmd cmd = {{0}};

{0}nob_cmd_append(&cmd, "cmake", "--build", "build", NULL);
{0}if (!nob_cmd_run_sync(cmd)) return (1);

{0}if (argc > 0)
{0}{{
{0}{0}const char *subcmd = nob_shift_args(&argc, &argv);

{0}{0}if (strcmp(subcmd, "run") == 0)
{0}{0}{{
{0}{0}{0}cmd.count = 0;
{0}{0}{0}nob_cmd_append(&cmd, "./build/{name}", NULL);
{0}{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);
{0}{0}}}
{0}{0}else if (strcmp(subcmd, "clean") == 0)
{0}{0}{{
{0}{0}{0}cmd.count = 0;
{0}{0}{0}nob_cmd_append(&cmd, "cmake", "--build", "build", "--target", "clean", NULL);
{0}{0}{0}if (!nob_cmd_run_sync(cmd)) return (1);
{0}{0}}}
{0}{0}else
{0}{0}{{
{0}{0}{0}nob_log(NOB_ERROR, "Subcomando desconocido: %s", subcmd);
{0}{0}{0}return (1);
{0}{0}}}
{0}}}

{0}return (0);
}}
"#,
"\t"
),
    )?;

write_readme(name, "C++")?;
write_gitignore_cpp()?;

// Build con CMake
run("cmake", &["-S", ".", "-B", "build"])?;
run("cmake", &["--build", "build"])?;

if git {
    init_git()?;
}

ok!("Proyecto C++ '{name}' creado con éxito.");
Ok(())
}

// ─── Generadores con herramientas nativas ─────────────────────────────────────

fn create_csharp_project(name: &str, git: bool) -> Result<(), String> {
    run("dotnet", &[
        "new", "console", "--language", "C#", "--name", name,
        "--output", ".", "--no-restore",
    ])?;
    write_file(".gitignore", "bin/\nobj/\n.vs/\n*.user\n")?;
    write_readme(name, "C#")?;
    run("dotnet", &["build"])?;
    if git {
        init_git()?;
    }
    ok!("Proyecto C# '{name}' creado con éxito.");
    Ok(())
}

fn create_rust_project(name: &str, git: bool) -> Result<(), String> {
    fs::create_dir_all("src").map_err(|e| format!("mkdir src: {e}"))?;
    // Own workspace: never add this project to an ancestor's Cargo workspace.
    write_file("Cargo.toml", &format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n\n[workspace]\n"
    ))?;
    write_file("src/main.rs", "fn main() {\n    println!(\"Hello, world!\");\n}\n")?;
    write_file(".gitignore", "/target/\n")?;
    write_readme(name, "Rust")?;
    run("cargo", &["build"])?;
    if git {
        init_git()?;
    }
    ok!("Proyecto Rust '{name}' creado con éxito.");
    Ok(())
}

fn create_python_project(name: &str, git: bool) -> Result<(), String> {
    run("uv", &[
        "init", "--app", "--no-package", "--name", name,
        "--python", ">=3.14,<3.15", "--no-pin-python", "--no-workspace",
        "--vcs", "none", ".",
    ])?;
    write_file(".python-version", "3.14\n")?;
    write_file(".gitignore", ".venv/\n__pycache__/\n*.py[cod]\n.pytest_cache/\n")?;
    write_readme(name, "Python")?;
    run("uv", &["sync", "--python", "3.14"])?;
    if git {
        init_git()?;
    }
    ok!("Proyecto Python 3.14 '{name}' creado con éxito.");
    Ok(())
}

// ─── Validación antes de escribir ─────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    C,
    Cpp,
    Csharp,
    Rust,
    Python,
}

impl Language {
    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_uppercase().as_str() {
            "C" => Ok(Self::C),
            "C++" | "CPP" => Ok(Self::Cpp),
            "C#" | "CS" | "CSHARP" => Ok(Self::Csharp),
            "RUST" | "RS" => Ok(Self::Rust),
            "PYTHON" | "PY" | "PYTHON3" | "PYTHON3.14" => Ok(Self::Python),
            other => Err(format!("Lenguaje desconocido: '{other}'. Usa C, C++, C#, Rust o Python.")),
        }
    }
}

fn validate_name(name: &str, lang: Language) -> Result<(), String> {
    if !name.starts_with(|c: char| c.is_ascii_alphabetic())
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("El nombre debe empezar por una letra ASCII y contener solo letras, números o '_'; no se admiten rutas.".into());
    }
    if matches!(lang, Language::C | Language::Cpp) && name.eq_ignore_ascii_case("main") {
        return Err("El nombre 'main' colisiona con el archivo de entrada de C/C++.".into());
    }
    Ok(())
}

fn ensure_empty_destination(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(format!("El destino {:?} existe y no es un directorio.", path));
    }
    let mut entries = fs::read_dir(path).map_err(|e| format!("No se pudo leer {:?}: {e}", path))?;
    if entries.next().is_some() {
        return Err(format!("El directorio {:?} no está vacío; usa un destino nuevo para evitar sobrescribir archivos.", path));
    }
    Ok(())
}

fn preflight(lang: Language, git: bool) -> Result<Option<PathBuf>, String> {
    let commands: &[&str] = match lang {
        Language::C => &["gcc", "make"],
        Language::Cpp => &["cmake"],
        Language::Csharp => &["dotnet"],
        Language::Rust => &["cargo"],
        Language::Python => &["uv"],
    };
    for cmd in commands {
        run(cmd, &["--version"])?;
    }
    if git {
        run("git", &["--version"])?;
    }
    if matches!(lang, Language::C | Language::Cpp) {
        let header = find_header("NOB_PATH", "nob.h")
            .ok_or("No se encontró nob.h. Define NOB_PATH o instálalo en /usr/local/share/InitProject/.")?;
        return header.canonicalize().map(Some)
            .map_err(|e| format!("No se pudo resolver {:?}: {e}", header));
    }
    Ok(None)
}

// ─── Helpers de escritura ─────────────────────────────────────────────────────

fn write_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Error escribiendo {path}: {e}"))?;
    ok!("Creado: {path}");
    Ok(())
}

fn write_readme(name: &str, lang: &str) -> Result<(), String> {
    let commands = match lang {
        "C" => "make\nmake run\nmake lib\n(cd test && python3 test.py)",
        "C++" => "cmake -S . -B build\ncmake --build build",
        "C#" => "dotnet build\ndotnet run",
        "Rust" => "cargo build\ncargo run\ncargo test",
        "Python" => "uv sync --python 3.14\nuv run python main.py\nuv add nombre_del_paquete",
        _ => unreachable!("unsupported README language"),
    };
    let details = match lang {
        "C++" => format!("\nEjecutar: `./build/{name}` (generador de configuración única).\n"),
        "C#" => "\nRequiere el SDK de .NET. El framework lo elige la plantilla del SDK instalado.\n".into(),
        "Python" => "\nRequiere uv. Python está fijado a la serie 3.14; uv puede descargarlo si no está instalado. Versiona `uv.lock` y `.python-version`, no `.venv/`.\n".into(),
        "Rust" => "\nRequiere Rust y Cargo. Versiona `Cargo.lock`; este binario tiene su propio workspace.\n".into(),
        _ => String::new(),
    };
    write_file(
        "README.md",
        &format!(
            "# {name}\n\nProyecto {lang} generado con InitProject.\n\n## Uso\n\n```sh\n{commands}\n```\n{details}"
        ),
    )
}

fn write_gitignore_c() -> Result<(), String> {
    write_file(
        ".gitignore",
        "bin/\n*.o\ntest/libtest.so\nnob.h\n",
    )
}

fn write_gitignore_cpp() -> Result<(), String> {
    write_file(
        ".gitignore",
        "build/\nnob.h\n*.o\n",
    )
}

fn init_git() -> Result<(), String> {
    run("git", &["init"])?;
    run("git", &["branch", "-m", "main"])?;
    ok!("Repositorio git inicializado en rama 'main'.");
    Ok(())
}

// ─── main ─────────────────────────────────────────────────────────────────────

fn main() {
    let matches = Command::new("init_project")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("name")
            .help("Nombre del proyecto")
            .index(1),
        )
        .arg(
            Arg::new("lang")
            .help("Lenguaje: C | C++ (CPP) | C# (CS, CSHARP) | Rust (RS) | Python (PY, PYTHON3, PYTHON3.14)")
            .index(2),
        )
        .arg(
            Arg::new("no-git")
            .long("no-git")
            .action(clap::ArgAction::SetTrue)
            .help("No inicializar repositorio git"),
        )
        .get_matches();

    let no_git = matches.get_flag("no-git");

    // ── Modo interactivo si faltan argumentos ──────────────────────────────
    let name = match matches.get_one::<String>("name") {
        Some(n) => n.clone(),
        None => {
            info!("Modo interactivo — no se pasaron argumentos");
            prompt("Nombre del proyecto:")
        }
    };

    let lang_raw = match matches.get_one::<String>("lang") {
        Some(l) => l.clone(),
        None => prompt("Lenguaje (C | C++ | C# | Rust | Python):"),
    };

    let lang = Language::parse(&lang_raw).unwrap_or_else(|e| {
        err!("{}", e);
        process::exit(1);
    });

    if let Err(e) = validate_name(&name, lang) {
        err!("{}", e);
        process::exit(1);
    }

    // Validate tools and resolve the header before creating/changing directories.
    let nob_src = preflight(lang, !no_git).unwrap_or_else(|e| {
        err!("{}", e);
        process::exit(1);
    });

    if let Err(e) = ensure_empty_destination(Path::new(&name)) {
        err!("{}", e);
        process::exit(1);
    }

    // ── Crear y entrar al directorio ──────────────────────────────────────
    fs::create_dir_all(&name).unwrap_or_else(|e| {
        err!("No se pudo crear el directorio '{}': {}", name, e);
        process::exit(1);
    });

    std::env::set_current_dir(&name).unwrap_or_else(|e| {
        err!("No se pudo entrar en '{}': {}", name, e);
        process::exit(1);
    });

    info!("Creando proyecto '{}' en {:?}...", name, std::env::current_dir().unwrap());

    // ── Despachar al generador correcto ──────────────────────────────────
    let result = match lang {
        Language::C => create_c_project(&name, !no_git, nob_src.as_deref().unwrap()),
        Language::Cpp => create_cpp_project(&name, !no_git, nob_src.as_deref().unwrap()),
        Language::Csharp => create_csharp_project(&name, !no_git),
        Language::Rust => create_rust_project(&name, !no_git),
        Language::Python => create_python_project(&name, !no_git),
    };

    if let Err(e) = result {
        err!("{}", e);
        process::exit(1);
    }
}
