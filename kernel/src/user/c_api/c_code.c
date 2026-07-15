extern int rust_add_numbers(int a, int b);

int c_add_numbers(int a, int b) {
    return rust_add_numbers(a, b);
}