#include <init.h>

using namespace std;

string	ft_toupper(const string &input)
{
	string	result;
	for (char c : input)
		result.push_back(toupper(c));
	return (result);
}

void	c(std::string name)
{
// Makefile
	ofstream makefile("Makefile");

	if (!makefile)
	{
		cerr << "Failed to created Makefile." << endl;
		exit(1);
	}

	makefile << R"(SRDIR = src/
SRC = $(wildcard $(SRDIR)*.c)
NAME = bin/)" << name << R"(
DSRC = $(addprefix $(SRDIR), $(SRC))
OBJS = $(SRC:.c=.o)
CC = gcc
CFLAGS = -Wall -Wextra -Werror -Iinclude
RM = rm -rf

all: $(NAME)

$(SRDIR)%.o: $(SRDIR)%.c
	$(CC) $(CFLAGS) -c $< -o $@
$(NAME): $(OBJS)
	$(CC) $(CFLAGS) -o $(NAME) $(OBJS)

clean:
	$(RM) $(OBJS)

fclean: clean
	$(RM) $(NAME)

run:
	./$(NAME)

re: fclean all

.PHONY: all clean fclean re)";
	makefile.close();

// Archivo C
	ofstream cFile("src/" + name + ".c");

	if (!cFile)
	{
		cerr << "Failed to created C File." << endl;
		exit(1);
	}

	cFile << R"(#include <)" << name << R"(.h>

int	main(int argc, char **argv)
{
	if (argc && argv)
		printf("Done C project!!\n");
	return (0);
})";
	cFile.close();

// Archivo H
	ofstream hFile("include/" + name + ".h");

	if (!hFile)
	{
		cerr << "Failed to created header File." << endl;
		exit(1);
	}

	hFile << R"(#ifndef )" << ft_toupper(name) << R"(_H
#define )" << ft_toupper(name) << R"(_H

#include <stdio.h>
#include <unistd.h>

#define ERROR_EXIT(...) fprintf(stderr, __VA_ARGS__); exit(1)
#define ERROR_RETURN(R, ...) fprintf(stderr, __VA_ARGS__); return R

#endif)";
	hFile.close();

// README
	ofstream readme("README.md");

	if (!readme)
		cerr << "Failed to created README file." << endl;

	readme.close();

// GITIGNORE
	ofstream gitignore(".gitignore");

	if (!makefile)
		cerr << "Failed to created gitignore file." << endl;

	gitignore.close();
}

void	cpp(std::string name)
{
// Makefile
	ofstream makefile("Makefile");

	if (!makefile)
	{
		cerr << "Failed to created Makefile." << endl;
		exit(1);
	}

	makefile << R"(SRDIR = src/
SRC = $(wildcard $(SRDIR)*.cpp)
NAME = bin/)" << name << R"(
DSRC = $(addprefix $(SRDIR), $(SRC))
OBJS = $(SRC:.cpp=.o)
CC = g++
CFLAGS = -Wall -Wextra -Werror -Iinclude
RM = rm -rf

all: $(NAME)

$(SRDIR)%.o: $(SRDIR)%.cpp
	$(CC) $(CFLAGS) -c $< -o $@
$(NAME): $(OBJS)
	$(CC) $(CFLAGS) -o $(NAME) $(OBJS)

clean:
	$(RM) $(OBJS)

fclean: clean
	$(RM) $(NAME)

run:
	./$(NAME)

re: fclean all

.PHONY: all clean fclean re)";
	makefile.close();

// Archivo CPP
	ofstream cFile("src/" + name + ".cpp");

	if (!cFile)
	{
		cerr << "Failed to created C++ File." << endl;
		exit(1);
	}

	cFile << R"(#include <)" << name << R"(.h>

int	main(int argc, char **argv)
{
	if (argc && argv)
		std::cout << "Done C++ project!!" << std::endl;
	return (0);
})";
	cFile.close();

// Archivo H
	ofstream hFile("include/" + name + ".h");

	if (!hFile)
	{
		cerr << "Failed to created header File." << endl;
		exit(1);
	}

	hFile << R"(#ifndef )" << ft_toupper(name) << R"(_H
#define )" << ft_toupper(name) << R"(_H

#include <iostream>
#include <unistd.h>

#define ERROR_EXIT(...) fprintf(stderr, __VA_ARGS__); exit(1)
#define ERROR_RETURN(R, ...) fprintf(stderr, __VA_ARGS__); return R

#endif)";
	hFile.close();

// README
	ofstream readme("README.md");

	if (!readme)
		cerr << "Failed to created README file." << endl;

	readme.close();

// GITIGNORE
	ofstream gitignore(".gitignore");

	if (!makefile)
		cerr << "Failed to created gitignore file." << endl;

	gitignore.close();
}
