# Confy

Manage values that appear in multiple configuration files in a centralized way.

## What

`Confy` is a small rust program that solves a recurring problem I had: changing the theme of multiple heterogeneous apps from a unique file.

`Confy` may be used for much more than that tought, it's up to you to be creative.

## How

You start `confy` by passing it a yaml config file (-c) and optionnaly watch-modes. The config file contains two sections: bindings and variables.

Bindings describe the input and output file to manage.
Variables are key-value pairs which you want to be substituted in from your input files into your output files.

Here is a sample configuration:

    bindings:
        -   from: some_app/some_config.cfy
            to: some_app/some_config.conf
        -   from: some_path/awesome.lua.cfy
            to: some_path/awesome.lua
    variables:
        terminal: termite
        some_usefull_concept: value
        color.0: #abcdef
        color.1: #abcabc
        ...
        color.15: #ffaa00
        color.primary: "@color.0"
        color.secondary: "@color.8"
        color.warning: "@color.5"

(Values starting with `@` will be dereferenced to the appropriate key when possible.)

This sample file tells us that we want to bind `some_app/some_config.cfy` to `some_app/some_config.conf` and replace `color.1` with `#abcabc` for instance.

A sample input file may be:

    ...
    theme.color.primary = "${{color.primary}}"
    theme.color.warning = "${{color.warning}}"
    ...

Which will result in something like that:

    ...
    theme.color.primary = "#abcdef"
    theme.color.warning = "#aaabbb"
    ...

As you can see, keys are surrounded with `${{` and `}}` to aleviate conflicts. It is often wise to surround the whole with quotes in orderd to avoid syntax errors while editing or when running.

## Watch Modes

If you simply run `confy -c some_config.yaml`, `confy` will stop after its job is done (output files generated).

You may, however, wish to regenerate an output file when its input file is modified without running the command again. You can use the --watch-bindings flag (-B) for this.

You may as well wish to regenerate all output files when the config file is modified. Use --watch-config (-C) for this.

Those flags will run the program as a daemon, watching for inotify events and reacting accordingly.
