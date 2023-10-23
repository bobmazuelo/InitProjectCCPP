#include <init.h>

using namespace std;

int     main(int argc, char **argv)
{
        if (argc != 3)
	{
error:
                cerr << "Usage: " << argv[0] << " [project name] " << "[C | C++ | CPP]" << endl;
		return (1);
	}

	string	name = argv[1];
	string	langC = argv[2];

	mkdir(name.c_str(), 0777);
	chdir(name.c_str());

	mkdir("src", 0777);
	mkdir("include", 0777);
	mkdir("bin", 0777);

	if (langC == "C")
		c(name);
	else if (langC == "C++" || langC == "CPP")
		cpp(name);
	else
		goto error;

	system("git init");
	system("git branch -m main");
	system("make all");
	system("make clean");
	system("make run");

        return (0);
}
