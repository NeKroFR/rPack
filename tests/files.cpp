#include <iostream>
#include <fstream>
#include <string>

int main() {
    std::ofstream file("/tmp/test.txt");
    file << "Hello, World!";
    file.close();

    std::ifstream file2("/tmp/test.txt");
    std::string content;
    if (file2) {
        std::getline(file2, content); // Read the entire line
    }
    std::cout << content << std::endl;
    file2.close();
    return 0;
}
