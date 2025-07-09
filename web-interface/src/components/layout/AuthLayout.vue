<template>
  <div class="auth-layout">
    <div class="auth-container">
      <div class="auth-header">
        <div class="auth-logo">
          <el-icon size="40" color="#409EFF">
            <DataBoard />
          </el-icon>
          <h1>DataMesh</h1>
        </div>
        <p class="auth-subtitle">Distributed Storage System</p>
      </div>
      
      <div class="auth-content">
        <router-view />
      </div>
      
      <div class="auth-footer">
        <p>&copy; 2024 DataMesh. All rights reserved.</p>
        <div class="auth-links">
          <a href="#" @click.prevent>Privacy Policy</a>
          <a href="#" @click.prevent>Terms of Service</a>
          <a href="#" @click.prevent>Help</a>
        </div>
      </div>
    </div>
    
    <div class="auth-background">
      <div class="auth-pattern"></div>
    </div>
  </div>
</template>

<script>
export default {
  name: 'AuthLayout'
}
</script>

<style scoped>
.auth-layout {
  min-height: 100vh;
  display: flex;
  position: relative;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.auth-container {
  width: 100%;
  max-width: 400px;
  margin: auto;
  padding: 40px 24px;
  background: var(--el-bg-color);
  border-radius: 16px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  z-index: 1;
}

.auth-header {
  text-align: center;
  margin-bottom: 32px;
}

.auth-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 8px;
}

.auth-logo h1 {
  margin: 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--el-text-color-primary);
}

.auth-subtitle {
  margin: 0;
  color: var(--el-text-color-secondary);
  font-size: 16px;
}

.auth-content {
  margin-bottom: 32px;
}

.auth-footer {
  text-align: center;
  padding-top: 24px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.auth-footer p {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.auth-links {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.auth-links a {
  font-size: 14px;
  color: var(--el-color-primary);
  text-decoration: none;
  transition: color 0.2s ease;
}

.auth-links a:hover {
  color: var(--el-color-primary-light-3);
}

.auth-background {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  overflow: hidden;
}

.auth-pattern {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image: 
    radial-gradient(circle at 20% 50%, rgba(255, 255, 255, 0.1) 0%, transparent 50%),
    radial-gradient(circle at 80% 20%, rgba(255, 255, 255, 0.1) 0%, transparent 50%),
    radial-gradient(circle at 40% 80%, rgba(255, 255, 255, 0.1) 0%, transparent 50%);
  animation: float 6s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-20px);
  }
}

/* Mobile responsive */
@media (max-width: 768px) {
  .auth-container {
    margin: 16px;
    padding: 32px 20px;
    max-width: none;
  }
  
  .auth-logo h1 {
    font-size: 24px;
  }
  
  .auth-subtitle {
    font-size: 14px;
  }
  
  .auth-links {
    flex-direction: column;
    gap: 8px;
  }
}
</style>