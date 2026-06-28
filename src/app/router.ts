import { createRouter, createWebHashHistory } from 'vue-router'
import DevTinyWorkbench from '../features/devtiny/project/DevTinyWorkbench.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [{ path: '/', name: 'workbench', component: DevTinyWorkbench }]
})
