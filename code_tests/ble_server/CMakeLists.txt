cmake_minimum_required(VERSION 3.20.0)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
# Pull in the Zephyr build system
find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})

project(ble_server)

# Export compile commands for clangd (Neovim)

target_sources(app PRIVATE src/main.c)
