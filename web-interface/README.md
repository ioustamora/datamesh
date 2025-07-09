# DataMesh Web Interface

A modern Vue.js web interface for the DataMesh distributed storage system, implementing the specifications from the DataMesh Application & Network Improvements Roadmap.

## 🚀 Features

### Core Functionality
- **File Management**: Upload, download, search, and organize files
- **Real-time Monitoring**: Live system metrics and file operations
- **Governance Interface**: Network governance, operator management, and voting
- **Administration**: User management, system configuration, and audit logs
- **Analytics Dashboard**: Performance metrics and usage analytics

### Technical Features
- **Vue 3 + Composition API**: Modern reactive framework
- **Element Plus**: Professional UI component library
- **Pinia**: State management with TypeScript support
- **WebSocket Integration**: Real-time updates and notifications
- **Responsive Design**: Mobile-first approach with adaptive layouts
- **Dark/Light Theme**: Automatic theme switching with custom colors
- **Progressive Web App**: Offline capabilities and app-like experience

## 🏗️ Architecture

### Project Structure
```
src/
├── components/           # Reusable Vue components
│   ├── layout/          # Layout components (MainLayout, AuthLayout)
│   ├── common/          # Common components (LoadingOverlay, etc.)
│   ├── dashboard/       # Dashboard-specific components
│   ├── files/           # File management components
│   ├── governance/      # Governance interface components
│   └── admin/           # Administration components
├── views/               # Page components
│   ├── auth/            # Authentication pages
│   ├── governance/      # Governance pages
│   ├── administration/  # Admin pages
│   └── errors/          # Error pages
├── store/               # Pinia state management
│   ├── auth.js          # Authentication store
│   ├── files.js         # File management store
│   ├── governance.js    # Governance store
│   ├── theme.js         # Theme management
│   ├── websocket.js     # WebSocket store
│   └── loading.js       # Loading state management
├── services/            # API services
│   └── api.js           # Axios-based API client
├── utils/               # Utility functions
├── assets/              # Static assets
└── router/              # Vue Router configuration
```

### State Management
The application uses Pinia for state management with the following stores:

- **Auth Store**: User authentication, authorization, and profile management
- **Files Store**: File operations, upload queue, and search functionality
- **Governance Store**: Network governance, operators, and proposals
- **Theme Store**: Theme switching, custom colors, and responsive breakpoints
- **WebSocket Store**: Real-time communication and event handling
- **Loading Store**: Global loading states and progress tracking

## 📊 Governance Interface

### Network Governance Dashboard
Based on the roadmap specifications, the governance interface provides:

- **Operator Management**: Register, monitor, and manage bootstrap operators
- **Network Health**: Real-time network status and consensus monitoring
- **Proposal System**: Submit and vote on governance proposals
- **Voting Interface**: Democratic decision-making with weighted voting
- **Admin Actions**: Execute administrative actions with proper authorization

### Bootstrap Operator Administration
- **Service Registration**: Register storage, bandwidth, and relay services
- **Operator Metrics**: Performance monitoring and reputation scoring
- **Stake Management**: Economic stake and governance weight calculation
- **Jurisdiction Compliance**: Legal jurisdiction and regulatory compliance

## 🔧 API Integration

### REST API Client
The web interface integrates with the DataMesh REST API through a comprehensive API client:

```javascript
// File operations
await filesAPI.uploadFile(formData)
await filesAPI.downloadFile(fileKey)
await filesAPI.searchFiles(query)

// Governance operations
await governanceAPI.getNetworkHealth()
await governanceAPI.registerOperator(operatorData)
await governanceAPI.executeAdminAction(actionData)

// Administration
await adminAPI.getUsers()
await adminAPI.updateUserQuota(userId, quota)
await adminAPI.getSystemHealth()
```

### WebSocket Integration
Real-time updates for:
- File upload/download progress
- Network health changes
- Governance updates
- Admin actions
- System notifications

## 🎨 User Interface

### Design System
- **Element Plus**: Professional component library with DataMesh theming
- **Responsive Grid**: Adaptive layouts for desktop, tablet, and mobile
- **Dark/Light Theme**: Automatic theme detection with manual override
- **Custom Colors**: Configurable primary colors with generated palettes
- **Accessibility**: WCAG 2.1 AA compliance with keyboard navigation

### Key Components
- **MainLayout**: Primary application layout with sidebar navigation
- **Dashboard**: Overview with quick stats and recent activity
- **FileManager**: Comprehensive file management interface
- **GovernancePanel**: Network governance and operator management
- **AdminPanel**: System administration and user management
- **AnalyticsDashboard**: Performance metrics and visualizations

## 📱 Mobile Support

### Responsive Features
- **Adaptive Navigation**: Collapsible sidebar for mobile devices
- **Touch Optimization**: Touch-friendly interactions and gestures
- **Mobile-first Design**: Optimized for small screens
- **Progressive Web App**: Installable web app with offline capabilities

## 🔒 Security

### Authentication & Authorization
- **JWT Token Management**: Secure token storage and refresh
- **Role-based Access**: Admin, operator, and user permissions
- **Session Management**: Automatic logout and session validation
- **CSRF Protection**: Cross-site request forgery prevention

### Data Protection
- **Secure API Communication**: HTTPS and request signing
- **Input Validation**: Client-side and server-side validation
- **XSS Prevention**: Content sanitization and CSP headers
- **Privacy Controls**: User data export and deletion

## 🚀 Getting Started

### Prerequisites
- Node.js 18+ and npm/yarn
- DataMesh API server running on port 8080
- Modern web browser with JavaScript enabled

### Installation
```bash
# Clone the repository
git clone https://github.com/datamesh/datamesh.git
cd datamesh/web-interface

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

### Configuration
```javascript
// vite.config.js
export default defineConfig({
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
      '/ws': 'ws://localhost:8080'
    }
  }
})
```

### Environment Variables
```bash
# .env.local
VITE_API_BASE_URL=http://localhost:8080/api/v1
VITE_WS_URL=ws://localhost:8080/ws
VITE_APP_NAME=DataMesh
VITE_APP_VERSION=1.0.0
```

## 🧪 Testing

### Unit Testing
```bash
# Run unit tests
npm run test

# Run with coverage
npm run test:coverage

# Watch mode
npm run test:watch
```

### End-to-End Testing
```bash
# Run E2E tests
npm run test:e2e

# Run in headless mode
npm run test:e2e:headless
```

## 📦 Deployment

### Production Build
```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

### Docker Deployment
```dockerfile
FROM node:18-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

## 🔧 Development

### Code Style
- **ESLint**: JavaScript/Vue linting with recommended rules
- **Prettier**: Code formatting with consistent style
- **Vue Style Guide**: Following Vue.js official style guide
- **TypeScript**: Optional TypeScript support for better type safety

### Development Commands
```bash
# Start development server
npm run dev

# Lint code
npm run lint

# Format code
npm run format

# Type check
npm run type-check
```

## 📚 Documentation

### API Documentation
- **OpenAPI/Swagger**: Auto-generated API documentation
- **Component Documentation**: Vue component documentation with examples
- **Store Documentation**: Pinia store documentation with usage examples

### User Guide
- **Getting Started**: Quick start guide for new users
- **Feature Guide**: Detailed feature documentation
- **Admin Guide**: Administration and governance documentation
- **Troubleshooting**: Common issues and solutions

## 🤝 Contributing

### Development Workflow
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new features
5. Submit a pull request

### Code Standards
- Follow Vue.js style guide
- Write comprehensive tests
- Document new features
- Maintain backward compatibility

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Vue.js Team**: For the excellent framework
- **Element Plus**: For the comprehensive UI components
- **DataMesh Community**: For feedback and contributions
- **Open Source Libraries**: All the amazing libraries that make this possible

---

For more information, visit the [DataMesh Documentation](https://docs.datamesh.io) or join our [Community Discord](https://discord.gg/datamesh).