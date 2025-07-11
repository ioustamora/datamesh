# Shell Completion Installation

## Bash
Copy the bash completion file to your bash completions directory:
```bash
# On most Linux systems:
sudo cp datamesh.bash /etc/bash_completion.d/
# Or user-specific:
mkdir -p ~/.local/share/bash-completion/completions
cp datamesh.bash ~/.local/share/bash-completion/completions/datamesh
```

## Zsh
Copy the zsh completion file to your zsh completions directory:
```bash
# Add to your .zshrc:
fpath=(~/.local/share/zsh/completions $fpath)
autoload -U compinit && compinit

# Copy the completion file:
mkdir -p ~/.local/share/zsh/completions
cp _datamesh ~/.local/share/zsh/completions/
```

## Fish
Copy the fish completion file to your fish completions directory:
```bash
mkdir -p ~/.config/fish/completions
cp datamesh.fish ~/.config/fish/completions/
```

## Testing
After installation, restart your shell or source your configuration file.
Test completions by typing:
```bash
datamesh <TAB>
datamesh file <TAB>
datamesh network <TAB>
```
