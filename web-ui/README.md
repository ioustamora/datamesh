# DataMesh Web UI

A modern, responsive web interface for the DataMesh distributed storage system built with React, TypeScript, and Tailwind CSS.

## Features

### 🎨 Modern UI/UX
- **Responsive Design**: Mobile-first approach with responsive layouts
- **Dark/Light Mode**: System-aware theme switching with manual override
- **Component Library**: Reusable UI components built with Tailwind CSS
- **Accessibility**: WCAG-compliant design with keyboard navigation support

### 🔐 Authentication & Authorization
- **JWT-based Authentication**: Secure token-based authentication
- **Role-based Access**: Different access levels (free, premium, enterprise)
- **Session Management**: Auto-refresh tokens and secure logout
- **Protected Routes**: Route-level authentication guards

### 📁 File Management
- **Drag & Drop Upload**: Intuitive file upload interface
- **Progress Tracking**: Real-time upload/download progress
- **File Preview**: Built-in file type detection and icons
- **Metadata Management**: Tags, descriptions, and file organization
- **Search & Filter**: Advanced file search with multiple criteria

### 🌐 Real-time Updates
- **WebSocket Integration**: Real-time status updates and notifications
- **Live Metrics**: System performance and network health monitoring
- **Push Notifications**: Instant alerts for important events
- **Connection Management**: Auto-reconnection with exponential backoff

### 📊 Analytics Dashboard
- **System Metrics**: CPU, memory, disk usage monitoring
- **Storage Analytics**: Capacity planning and usage statistics
- **Network Health**: Peer connectivity and performance metrics
- **Interactive Charts**: Data visualization with Recharts

### ⚙️ Configuration
- **User Settings**: Personalized preferences and configurations
- **Theme Management**: Light/dark/system theme switching
- **Notification Settings**: Customizable alert preferences
- **Account Management**: Profile updates and security settings

## Tech Stack

### Frontend Framework
- **React 18**: Latest React with concurrent features
- **TypeScript**: Type-safe development
- **Vite**: Fast build tool and dev server
- **React Router**: Client-side routing

### State Management
- **Zustand**: Lightweight state management
- **React Query**: Server state management and caching
- **Persist Middleware**: Local storage persistence

### UI & Styling
- **Tailwind CSS**: Utility-first CSS framework
- **Headless UI**: Unstyled, accessible UI components
- **Heroicons**: Beautiful SVG icons
- **Framer Motion**: Smooth animations and transitions

### Data & API
- **Axios**: HTTP client with interceptors
- **Socket.io**: WebSocket client for real-time updates
- **React Hook Form**: Efficient form handling
- **React Hot Toast**: Elegant notifications

### Development Tools
- **ESLint**: Code linting and formatting
- **TypeScript**: Static type checking
- **PostCSS**: CSS processing
- **Autoprefixer**: CSS vendor prefixing

## Project Structure

```
src/
├── components/          # Reusable UI components
│   ├── ui/             # Basic UI elements (buttons, inputs, etc.)
│   ├── Layout.tsx      # Main layout wrapper
│   └── ProtectedRoute.tsx # Authentication guard
├── pages/              # Page components
│   ├── Dashboard.tsx   # Main dashboard
│   ├── Files.tsx       # File management
│   ├── Upload.tsx      # File upload interface
│   ├── Network.tsx     # Network monitoring
│   ├── Login.tsx       # Authentication
│   └── Settings.tsx    # User settings
├── hooks/              # Custom React hooks
│   └── useWebSocket.ts # WebSocket integration
├── stores/             # State management
│   ├── authStore.ts    # Authentication state
│   └── themeStore.ts   # Theme preferences
├── types/              # TypeScript type definitions
├── utils/              # Utility functions
│   ├── api.ts          # API client configuration
│   └── cn.ts           # Tailwind utilities
└── styles/             # Global styles
    └── globals.css     # Tailwind base styles
```

## Getting Started

### Prerequisites
- Node.js 18+
- npm or yarn
- DataMesh API server running on port 8080

### Installation

1. **Install dependencies**:
   ```bash
   cd web-ui
   npm install
   ```

2. **Start development server**:
   ```bash
   npm run dev
   ```

3. **Open in browser**:
   Navigate to `http://localhost:3000`

### Development Commands

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Run linting
npm run lint

# Type checking
npm run type-check
```

## Configuration

### Environment Variables
Create a `.env.local` file for local development:

```env
VITE_API_URL=http://localhost:8080/api/v1
VITE_WS_URL=ws://localhost:8080/ws
```

### API Integration
The UI automatically proxies API requests to the DataMesh server:
- REST API: `http://localhost:8080/api/v1`
- WebSocket: `ws://localhost:8080/ws`

## Features Implementation Status

### ✅ Completed
- [x] Authentication system with JWT
- [x] Responsive layout with dark/light theme
- [x] Dashboard with real-time metrics
- [x] WebSocket integration for live updates
- [x] File type detection and icons
- [x] Progress tracking components
- [x] Error handling and notifications
- [x] State management with persistence

### 🚧 In Progress
- [ ] Complete file management interface
- [ ] Advanced search and filtering
- [ ] Governance proposal system
- [ ] Network topology visualization
- [ ] Settings management UI

### 📋 Planned
- [ ] File preview capabilities
- [ ] Batch operations interface
- [ ] Analytics and reporting
- [ ] Mobile app (React Native)
- [ ] Offline support with service workers

## API Endpoints Used

### Authentication
- `POST /auth/login` - User authentication
- `POST /auth/register` - User registration
- `GET /auth/me` - Get current user
- `POST /auth/logout` - Logout user

### Files
- `POST /files` - Upload file
- `GET /files` - List files
- `GET /files/:id` - Download file
- `DELETE /files/:id` - Delete file
- `GET /files/:id/metadata` - Get file metadata

### Analytics
- `GET /analytics/system` - System metrics
- `GET /analytics/storage` - Storage metrics
- `GET /analytics/network` - Network metrics

### WebSocket Events
- `FileUploadProgress` - Upload progress updates
- `FileDownloadProgress` - Download progress updates
- `SystemStatus` - System status changes
- `NetworkHealth` - Network health updates
- `GovernanceUpdate` - Governance events

## Security Features

### Authentication Security
- JWT token-based authentication
- Automatic token refresh
- Secure logout with token invalidation
- Route-level access control

### Data Security
- XSS protection with CSP headers
- CSRF protection
- Secure cookie handling
- Input validation and sanitization

### Network Security
- HTTPS enforcement in production
- WebSocket connection security
- API request/response validation
- Error message sanitization

## Performance Optimizations

### Bundle Optimization
- Code splitting by route and vendor
- Tree shaking for unused code
- Asset optimization and compression
- Lazy loading for non-critical components

### Runtime Performance
- React Query for efficient data fetching
- Virtualization for large lists
- Debounced search inputs
- Optimized re-renders with React.memo

### Caching Strategy
- Browser caching for static assets
- API response caching with React Query
- Service worker for offline support
- Local storage for user preferences

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Contributing

1. Follow the existing code style and patterns
2. Add TypeScript types for all new interfaces
3. Include tests for new functionality
4. Update documentation for API changes
5. Ensure responsive design works on all screen sizes

## License

This project is part of the DataMesh distributed storage system.