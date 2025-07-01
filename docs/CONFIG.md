<div align="center">
    <img src="https://firebasestorage.googleapis.com/v0/b/devaloop-labs.firebasestorage.app/o/devalang-teal-logo.svg?alt=media&token=d2a5705a-1eba-4b49-88e6-895a761fb7f7" alt="Devalang Logo">
</div>

# Devalang Configuration File

Use a configuration file if you don't want to pass command-line arguments every time you run a command. The configuration file allows you to set default values for various settings, making it easier to manage your Devalang project.

## Ignoring the Configuration File

If you prefer not to use a configuration file, you can ignore it by passing the `--no-config` flag when running Devalang commands. This will bypass any settings defined in the configuration file and use only the command-line arguments you provide.

## Structure of the Configuration File

The configuration file is a TOML (Tom's Obvious, Minimal Language) file that contains key-value pairs to define various settings for your Devalang project. Below is a sample configuration file:

```toml
[defaults]
entry = "./src"
output = "./output"
watch = true
```

### Available Settings

- `entry`: (String) The entry point for your Devalang project
- `output`: (String) The output directory for generated files
- `watch`: (Boolean) Whether to watch for changes in files and automatically rebuild or check them
