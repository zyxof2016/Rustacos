const API_BASE = '/nacos/v1';

// 页面切换
function showPage(pageId) {
    // 隐藏所有页面
    document.querySelectorAll('.page-content').forEach(page => {
        page.classList.add('d-none');
    });
    
    // 显示选中的页面
    document.getElementById(pageId + '-page').classList.remove('d-none');
    
    // 更新导航栏状态
    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.remove('active');
    });
    event.target.classList.add('active');
    
    // 加载页面数据
    loadPageData(pageId);
}

// 加载页面数据
async function loadPageData(pageId) {
    switch(pageId) {
        case 'dashboard':
            await loadDashboardData();
            break;
        case 'services':
            await loadServicesData();
            break;
        case 'configs':
            await loadConfigsData();
            break;
        case 'namespaces':
            await loadNamespacesData();
            break;
    }
}

// 加载仪表盘数据
async function loadDashboardData() {
    try {
        // 加载统计数据
        const [servicesResponse, instancesResponse, configsResponse, namespacesResponse] = await Promise.all([
            fetch(`${API_BASE}/ns/service/list`),
            fetch(`${API_BASE}/ns/instance/list`),
            fetch(`${API_BASE}/cs/configs/list?namespace=public`),
            fetch(`${API_BASE}/console/namespaces`)
        ]);
        
        const services = await servicesResponse.json();
        const instances = await instancesResponse.json();
        const configs = await configsResponse.json();
        const namespaces = await namespacesResponse.json();
        
        document.getElementById('total-services').textContent = services.data?.length || 0;
        document.getElementById('total-instances').textContent = instances.data?.length || 0;
        document.getElementById('total-configs').textContent = configs.data?.length || 0;
        document.getElementById('total-namespaces').textContent = namespaces.data?.length || 0;
        
        // 加载最近服务
        const recentServices = document.getElementById('recent-services');
        if (services.data && services.data.length > 0) {
            recentServices.innerHTML = services.data.slice(0, 5).map(service => 
                `<div class="list-group-item">
                    <div class="d-flex w-100 justify-content-between">
                        <h6 class="mb-1">${service}</h6>
                        <small>运行中</small>
                    </div>
                </div>`
            ).join('');
        } else {
            recentServices.innerHTML = '<div class="text-muted">暂无服务</div>';
        }
        
        // 设置启动时间
        document.getElementById('start-time').textContent = new Date().toLocaleString();
        
    } catch (error) {
        console.error('加载仪表盘数据失败:', error);
    }
}

// 加载服务数据
async function loadServicesData() {
    try {
        const response = await fetch(`${API_BASE}/ns/instance/list`);
        const result = await response.json();
        
        const servicesTable = document.getElementById('services-table');
        if (result.data && result.data.length > 0) {
            servicesTable.innerHTML = result.data.map(instance => 
                `<tr>
                    <td>${instance.service_name}</td>
                    <td>${instance.id}</td>
                    <td>${instance.ip}</td>
                    <td>${instance.port}</td>
                    <td>${instance.group_name}</td>
                    <td>${instance.cluster_name}</td>
                    <td><span class="badge bg-success">健康</span></td>
                    <td>
                        <button class="btn btn-sm btn-outline-danger" onclick="deregisterInstance('${instance.id}', '${instance.service_name}')">
                            <i class="bi bi-trash"></i>
                        </button>
                    </td>
                </tr>`
            ).join('');
        } else {
            servicesTable.innerHTML = '<tr><td colspan="8" class="text-center text-muted">暂无服务实例</td></tr>';
        }
    } catch (error) {
        console.error('加载服务数据失败:', error);
    }
}

// 加载配置数据
async function loadConfigsData() {
    try {
        // 先加载命名空间列表
        const namespacesResponse = await fetch(`${API_BASE}/console/namespaces`);
        const namespacesResult = await namespacesResponse.json();
        
        const namespaceSelect = document.getElementById('config-namespace');
        const namespaceModal = document.getElementById('config-namespace-modal');
        
        if (namespacesResult.data) {
            const options = namespacesResult.data.map(ns => 
                `<option value="${ns.namespace}">${ns.namespace_show_name}</option>`
            ).join('');
            
            namespaceSelect.innerHTML = '<option value="">选择命名空间</option>' + options;
            namespaceModal.innerHTML = options;
        }
        
        // 加载配置列表
        const namespace = namespaceSelect.value || 'public';
        const configsResponse = await fetch(`${API_BASE}/cs/configs/list?namespace=${namespace}`);
        const configsResult = await configsResponse.json();
        
        const configsTable = document.getElementById('configs-table');
        if (configsResult.data && configsResult.data.length > 0) {
            configsTable.innerHTML = configsResult.data.map(config => 
                `<tr>
                    <td>${config.data_id}</td>
                    <td>${config.group}</td>
                    <td>${config.namespace}</td>
                    <td>${new Date(config.update_time * 1000).toLocaleString()}</td>
                    <td>
                        <button class="btn btn-sm btn-outline-primary" onclick="viewConfig('${config.data_id}', '${config.group}', '${config.namespace}')">
                            <i class="bi bi-eye"></i>
                        </button>
                        <button class="btn btn-sm btn-outline-danger" onclick="removeConfig('${config.data_id}', '${config.group}', '${config.namespace}')">
                            <i class="bi bi-trash"></i>
                        </button>
                    </td>
                </tr>`
            ).join('');
        } else {
            configsTable.innerHTML = '<tr><td colspan="5" class="text-center text-muted">暂无配置</td></tr>';
        }
    } catch (error) {
        console.error('加载配置数据失败:', error);
    }
}

// 加载命名空间数据
async function loadNamespacesData() {
    try {
        const response = await fetch(`${API_BASE}/console/namespaces`);
        const result = await response.json();
        
        const namespacesTable = document.getElementById('namespaces-table');
        if (result.data && result.data.length > 0) {
            namespacesTable.innerHTML = result.data.map(namespace => 
                `<tr>
                    <td>${namespace.namespace}</td>
                    <td>${namespace.namespace_show_name}</td>
                    <td>${namespace.namespace_desc}</td>
                    <td>${namespace.quota}</td>
                    <td>${new Date(namespace.create_time * 1000).toLocaleString()}</td>
                    <td>
                        <button class="btn btn-sm btn-outline-primary" onclick="editNamespace('${namespace.namespace}', '${namespace.namespace_show_name}', '${namespace.namespace_desc}', ${namespace.quota})">
                            <i class="bi bi-pencil"></i>
                        </button>
                        ${namespace.namespace !== 'public' ? 
                            `<button class="btn btn-sm btn-outline-danger" onclick="deleteNamespace('${namespace.namespace}')">
                                <i class="bi bi-trash"></i>
                            </button>` : ''}
                    </td>
                </tr>`
            ).join('');
        } else {
            namespacesTable.innerHTML = '<tr><td colspan="6" class="text-center text-muted">暂无命名空间</td></tr>';
        }
    } catch (error) {
        console.error('加载命名空间数据失败:', error);
    }
}

// 注册服务
async function registerService() {
    const data = {
        ip: document.getElementById('service-ip').value,
        port: parseInt(document.getElementById('service-port').value),
        service_name: document.getElementById('service-name').value,
        group_name: document.getElementById('service-group').value,
        cluster_name: document.getElementById('service-cluster').value,
        weight: parseFloat(document.getElementById('service-weight').value)
    };
    
    try {
        const response = await fetch(`${API_BASE}/ns/instance`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            alert('服务注册成功');
            bootstrap.Modal.getInstance(document.getElementById('registerServiceModal')).hide();
            document.getElementById('registerServiceForm').reset();
            loadServicesData();
        } else {
            alert('服务注册失败');
        }
    } catch (error) {
        console.error('注册服务失败:', error);
        alert('注册服务失败');
    }
}

// 注销实例
async function deregisterInstance(instanceId, serviceName) {
    if (!confirm('确定要注销此实例吗？')) return;
    
    try {
        const response = await fetch(`${API_BASE}/ns/instance/${serviceName}/${instanceId}`, {
            method: 'DELETE'
        });
        
        if (response.ok) {
            alert('实例注销成功');
            loadServicesData();
        } else {
            alert('实例注销失败');
        }
    } catch (error) {
        console.error('注销实例失败:', error);
        alert('注销实例失败');
    }
}

// 发布配置
async function publishConfig() {
    const data = {
        data_id: document.getElementById('config-data-id').value,
        group: document.getElementById('config-group').value,
        namespace: document.getElementById('config-namespace-modal').value,
        content: document.getElementById('config-content').value
    };
    
    try {
        const response = await fetch(`${API_BASE}/cs/configs`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            alert('配置发布成功');
            bootstrap.Modal.getInstance(document.getElementById('publishConfigModal')).hide();
            document.getElementById('publishConfigForm').reset();
            loadConfigsData();
        } else {
            alert('配置发布失败');
        }
    } catch (error) {
        console.error('发布配置失败:', error);
        alert('发布配置失败');
    }
}

// 删除配置
async function removeConfig(dataId, group, namespace) {
    if (!confirm('确定要删除此配置吗？')) return;
    
    try {
        const response = await fetch(`${API_BASE}/cs/configs?data_id=${dataId}&group=${group}&namespace=${namespace}`, {
            method: 'DELETE'
        });
        
        if (response.ok) {
            alert('配置删除成功');
            loadConfigsData();
        } else {
            alert('配置删除失败');
        }
    } catch (error) {
        console.error('删除配置失败:', error);
        alert('删除配置失败');
    }
}

// 查看配置
async function viewConfig(dataId, group, namespace) {
    try {
        const response = await fetch(`${API_BASE}/cs/configs?data_id=${dataId}&group=${group}&namespace=${namespace}`);
        const result = await response.json();
        
        if (result.data) {
            alert(`配置内容：\n\n${result.data.content}`);
        } else {
            alert('配置不存在');
        }
    } catch (error) {
        console.error('查看配置失败:', error);
        alert('查看配置失败');
    }
}

// 创建命名空间
async function createNamespace() {
    const data = {
        namespace: document.getElementById('namespace-id').value,
        namespace_show_name: document.getElementById('namespace-show-name').value,
        namespace_desc: document.getElementById('namespace-desc').value
    };
    
    try {
        const response = await fetch(`${API_BASE}/console/namespaces`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            alert('命名空间创建成功');
            bootstrap.Modal.getInstance(document.getElementById('createNamespaceModal')).hide();
            document.getElementById('createNamespaceForm').reset();
            loadNamespacesData();
        } else {
            alert('命名空间创建失败');
        }
    } catch (error) {
        console.error('创建命名空间失败:', error);
        alert('创建命名空间失败');
    }
}

// 编辑命名空间
function editNamespace(namespace, showName, desc, quota) {
    const newShowName = prompt('显示名称:', showName);
    if (newShowName === null) return;
    
    const newDesc = prompt('描述:', desc);
    if (newDesc === null) return;
    
    const newQuota = prompt('配额:', quota);
    if (newQuota === null) return;
    
    updateNamespace(namespace, newShowName, newDesc, parseInt(newQuota));
}

// 更新命名空间
async function updateNamespace(namespace, showName, desc, quota) {
    const data = {
        namespace_show_name: showName,
        namespace_desc: desc,
        quota: quota
    };
    
    try {
        const response = await fetch(`${API_BASE}/console/namespaces/${namespace}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            alert('命名空间更新成功');
            loadNamespacesData();
        } else {
            alert('命名空间更新失败');
        }
    } catch (error) {
        console.error('更新命名空间失败:', error);
        alert('更新命名空间失败');
    }
}

// 删除命名空间
async function deleteNamespace(namespace) {
    if (!confirm(`确定要删除命名空间 "${namespace}" 吗？此操作将同时删除该命名空间下的所有配置！`)) return;
    
    try {
        const response = await fetch(`${API_BASE}/console/namespaces/${namespace}`, {
            method: 'DELETE'
        });
        
        if (response.ok) {
            alert('命名空间删除成功');
            loadNamespacesData();
        } else {
            alert('命名空间删除失败');
        }
    } catch (error) {
        console.error('删除命名空间失败:', error);
        alert('删除命名空间失败');
    }
}

// 页面加载时初始化
document.addEventListener('DOMContentLoaded', function() {
    loadDashboardData();
});