// Token 管理功能
/**
 * 保存认证令牌到本地存储
 * @param {string} token - 要保存的认证令牌
 * @returns {void}
 */
function saveAuthToken(token) {
  const expiryTime = new Date().getTime() + (24 * 60 * 60 * 1000); // 24小时后过期
  localStorage.setItem('authToken', token);
  localStorage.setItem('authTokenExpiry', expiryTime);
}

/**
 * 获取存储的认证令牌
 * @returns {string|null} 如果令牌有效则返回令牌，否则返回 null
 */
function getAuthToken() {
  const token = localStorage.getItem('authToken');
  const expiry = localStorage.getItem('authTokenExpiry');

  if (!token || !expiry) {
    return null;
  }

  if (new Date().getTime() > parseInt(expiry)) {
    localStorage.removeItem('authToken');
    localStorage.removeItem('authTokenExpiry');
    return null;
  }

  return token;
}

// 消息显示功能
/**
 * 在指定元素中显示消息
 * @param {string} elementId - 目标元素的 ID
 * @param {string} text - 要显示的消息文本
 * @param {boolean} [isError=false] - 是否为错误消息
 * @returns {void}
 */
function showMessage(elementId, text, isError = false) {
  let msg = document.getElementById(elementId);

  // 如果消息元素不存在，创建一个新的
  if (!msg) {
    msg = document.createElement('div');
    msg.id = elementId;
    document.body.appendChild(msg);
  }

  msg.className = `floating-message ${isError ? 'error' : 'success'}`;
  msg.innerHTML = text.replace(/\n/g, '<br>');
}

// 确保消息容器存在
/**
 * 确保消息容器存在于 DOM 中
 * @returns {HTMLElement} 消息容器元素
 */
function ensureMessageContainer() {
  let container = document.querySelector('.message-container');
  if (!container) {
    container = document.createElement('div');
    container.className = 'message-container';
    document.body.appendChild(container);
  }
  return container;
}

/**
 * 显示全局消息提示
 * @param {string} text - 要显示的消息文本
 * @param {boolean} [isError=false] - 是否为错误消息
 * @param {number} [timeout=3000] - 消息显示时长（毫秒）
 * @returns {void}
 */
function showGlobalMessage(text, isError = false, timeout = 3000) {
  const container = ensureMessageContainer();

  const msgElement = document.createElement('div');
  msgElement.className = `message ${isError ? 'error' : 'success'}`;
  msgElement.textContent = text;

  container.appendChild(msgElement);

  // 设置淡出动画和移除
  setTimeout(() => {
    msgElement.style.animation = 'messageOut 0.3s ease-in-out';
    setTimeout(() => {
      msgElement.remove();
      // 如果容器为空，也移除容器
      if (container.children.length === 0) {
        container.remove();
      }
    }, 300);
  }, timeout);
}

// Token 输入框自动填充和事件绑定
function initializeTokenHandling(inputId) {
  document.addEventListener('DOMContentLoaded', () => {
    const authToken = getAuthToken();
    if (authToken) {
      document.getElementById(inputId).value = authToken;
    }
  });

  document.getElementById(inputId).addEventListener('change', (e) => {
    if (e.target.value) {
      saveAuthToken(e.target.value);
    } else {
      localStorage.removeItem('authToken');
      localStorage.removeItem('authTokenExpiry');
    }
  });
}

// API 请求通用处理
async function makeAuthenticatedRequest(url, options = {}) {
  const tokenId = options.tokenId || 'authToken';
  const token = document.getElementById(tokenId).value;

  if (!token) {
    showGlobalMessage('请输入 AUTH_TOKEN', true);
    return null;
  }

  const defaultOptions = {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  };

  try {
    const response = await fetch(url, { ...defaultOptions, ...options });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    showGlobalMessage(`请求失败: ${error.message}`, true);
    return null;
  }
}

/**
 * 从字符串解析布尔值
 * @param {string} str - 要解析的字符串
 * @param {boolean|null} defaultValue - 解析失败时的默认值
 * @returns {boolean|null} 解析结果，如果无法解析则返回默认值
 */
function parseBooleanFromString(str, defaultValue = null) {
  if (typeof str !== 'string') {
    return defaultValue;
  }

  const lowercaseStr = str.toLowerCase().trim();

  if (lowercaseStr === 'true' || lowercaseStr === '1') {
    return true;
  } else if (lowercaseStr === 'false' || lowercaseStr === '0') {
    return false;
  } else {
    return defaultValue;
  }
}

/**
 * 将布尔值转换为字符串
 * @param {boolean|undefined|null} value - 要转换的布尔值
 * @param {string} defaultValue - 转换失败时的默认值
 * @returns {string} 转换结果，如果输入无效则返回默认值
 */
function parseStringFromBoolean(value, defaultValue = null) {
  if (typeof value !== 'boolean') {
    return defaultValue;
  }

  return value ? 'true' : 'false';
}

/**
 * 将会员类型代码转换为显示名称
 * @param {string|null} type - 会员类型代码,如 'free_trial', 'pro', 'free', 'enterprise' 等
 * @returns {string} 格式化后的会员类型显示名称
 * @example
 * formatMembershipType('free_trial') // 返回 'Pro Trial'
 * formatMembershipType('pro') // 返回 'Pro'
 * formatMembershipType(null) // 返回 '-'
 * formatMembershipType('custom_type') // 返回 'Custom Type'
 */
function formatMembershipType(type) {
  if (!type) return '-';
  switch (type) {
    case 'free_trial': return 'Pro Trial';
    case 'pro': return 'Pro';
    case 'free': return 'Free';
    case 'enterprise': return 'Business';
    default: return type
      .split('_')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }
}
