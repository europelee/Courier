# 前端编码规范（Vue3 + TypeScript）

## Vue3 组件规范

**只用 Composition API**，禁止 Options API：

```typescript
// ✅ 正确：Composition API
<script setup lang="ts">
import { ref, computed } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

// ❌ 禁止：Options API
export default {
  data() { return { count: 0 } },
  computed: { doubled() { return this.count * 2 } }
}
```

## TypeScript 规范

- 开启严格模式（`strict: true`）
- **禁止 `any` 类型**，用 `unknown` + 类型守卫代替
- 接口名用 `PascalCase`，不加 `I` 前缀

```typescript
// ✅ 正确
interface Tunnel {
  id: string
  subdomain: string
  status: 'active' | 'inactive'
}

// ❌ 禁止
const data: any = response.data
```

## API 调用规范

**所有 API 调用统一在 `src/api/tunnelApi.ts` 中定义**，组件不直接调用 `fetch` 或 `axios`：

```typescript
// ✅ 正确：在 src/api/tunnelApi.ts 中定义
export async function listTunnels(): Promise<Tunnel[]> {
  const response = await fetch('/api/v1/tunnels')
  if (!response.ok) throw new Error(`HTTP ${response.status}`)
  return response.json()
}

// 组件中调用
import { listTunnels } from '@/api/tunnelApi'
const tunnels = await listTunnels()

// ❌ 禁止：组件内直接写 fetch
const resp = await fetch('/api/v1/tunnels') // 禁止写在组件里
```

## 变量声明

- 优先用 `const`，只有确实需要重新赋值时才用 `let`
- 禁止 `var`

## 禁止提交的内容

- `console.log`、`console.error`（调试完毕后必须删除）
- 注释掉的死代码块
- `// TODO` 和 `// FIXME`（提交前必须处理或转为 issue）
