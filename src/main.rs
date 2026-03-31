use clap::{Arg, Command};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

// ─── Colores ANSI ─────────────────────────────────────────────────────────────
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
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
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("{}⚠  {}{}", YELLOW, format!($($arg)*), RESET)
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
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Junto al ejecutable
    if let Ok(exe) = std::env::current_exe() {
        let p = exe.parent().unwrap_or(Path::new(".")).join(filename);
        if p.exists() {
            return Some(p);
        }
    }

    // 3. /usr/local/share/InitProject/
    let system = PathBuf::from(format!("/usr/local/share/InitProject/{}", filename));
    if system.exists() {
        return Some(system);
    }

    // 4. Directorio de trabajo actual
    let cwd = std::env::current_dir()
        .unwrap_or_default()
        .join(filename);
    if cwd.exists() {
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

fn create_c_project(name: &str, git: bool) -> Result<(), String> {
    let upper = name.to_uppercase();

    // Directorios
    for dir in &["src", "include", "test", "bin"] {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
    }

    // nob.h
    let nob_src = find_header("NOB_PATH", "nob.h")
        .ok_or("ERROR @ No se encontró nob.h. Usa NOB_PATH o instálalo en /usr/local/share/InitProject/")?;
    copy_header(&nob_src, "nob.h")?;
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
CFLAGS  = -Wall -Wextra -Werror -Iinclude
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
        &format!("src/main.c"),
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

fn create_cpp_project(name: &str, git: bool) -> Result<(), String> {
    let upper = name.to_uppercase();

    for dir in &["src", "include", "test", "build"] {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {dir}: {e}"))?;
    }

    // nob.h
    let nob_src = find_header("NOB_PATH", "nob.h")
        .ok_or("ERROR @ No se encontró nob.h. Usa NOB_PATH o instálalo en /usr/local/share/InitProject/")?;
    copy_header(&nob_src, "nob.h")?;
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

// ─── Helpers de escritura ─────────────────────────────────────────────────────

fn write_file(path: &str, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Error escribiendo {path}: {e}"))?;
    ok!("Creado: {path}");
    Ok(())
}

fn write_readme(name: &str, lang: &str) -> Result<(), String> {
    write_file(
        "README.md",
        &format!(
            "# {name}\n\nProyecto {lang} generado con InitProject.\n\n## Build\n\nVer `Makefile` o `CMakeLists.txt`.\n"
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
        .version("2.1.0")
        .author("kodi2023")
        .about("Inicializa proyectos C, C++, C#, Rust con estructura estándar")
        .arg(
            Arg::new("name")
            .help("Nombre del proyecto")
            .index(1),
        )
        .arg(
            Arg::new("lang")
            .help("Lenguaje: C | C++ CPP")
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
        None => prompt("Lenguaje (C | C++ CPP):"),
    };

    let lang = lang_raw.trim().to_uppercase();

    if name.is_empty() {
        err!("El nombre del proyecto no puede estar vacío.");
        process::exit(1);
    }

    // ── Detección de directorio existente ────────────────────────────────
    if Path::new(&name).exists() {
        warn!("El directorio '{}' ya existe.", name);
        let answer = prompt("¿Continuar de todas formas? [s/N]:");
        if !matches!(answer.to_lowercase().as_str(), "s" | "si" | "sí" | "yes" | "y") {
            info!("Operación cancelada.");
            process::exit(0);
        }
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
    let result = match lang.as_str() {
        "C" => create_c_project(&name, !no_git),
        "C++" | "CPP" => create_cpp_project(&name, !no_git),
        other => {
            err!("Lenguaje desconocido: '{}'. Usa C, C++ o CPP.", other);
            process::exit(1);
        }
    };

    if let Err(e) = result {
        err!("{}", e);
        process::exit(1);
    }
}
