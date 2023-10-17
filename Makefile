SRDIR = srcs/
SRC = $(wildcard $(SRDIR)*.cpp)
NAME = bin/InitProject
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

.PHONY: all clean fclean re
