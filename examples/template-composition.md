# Template Composition Example

This example shows how to combine multiple templates for complex projects.

## Base Templates

Create individual component templates:

### Frontend Template (~/.local/masstemplate/frontend-react/)

```
frontend-react/
├── package.json
├── src/
│   ├── App.js
│   └── index.js
└── .mtem/
    └── post_install.sh  # Runs: npm install
```

### Backend Template (~/.local/masstemplate/backend-node/)

```
backend-node/
├── package.json
├── server.js
└── .mtem/
    └── post_install.sh  # Runs: npm install
```

## Composite Template (~/.local/masstemplate/fullstack-app/)

```
fullstack-app/
├── .mtem/
│   └── post_install.sh
└── README.md
```

### Post-Install Script (.mtem/post_install.sh)

```bash
#!/bin/bash
echo "Setting up fullstack application..."

# Apply component templates
mtem apply frontend-react --dest frontend --yes
mtem apply backend-node --dest backend --yes

# Additional setup
echo "Setting up development environment..."
# Create docker-compose.yml, shared configs, etc.

echo "Fullstack app ready! Run 'docker-compose up' to start development."
```

## Usage

```bash
mtem apply fullstack-app --dest my-fullstack-project
```

This creates:
```
my-fullstack-project/
├── frontend/     # React app
├── backend/      # Node.js API
├── docker-compose.yml
└── README.md     # Combined setup instructions
```

## Benefits

- **Modularity**: Reuse components across projects
- **Maintainability**: Update individual components independently
- **Flexibility**: Mix and match different stacks
- **Consistency**: Standardized setups for common patterns