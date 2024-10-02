export default {
  path: "/operation",
  redirect: "/operation/index",
  meta: {
    icon: "fileChartLine",
    title: "operation",
    rank: 2
  },
  children: [
    {
      path: "/operation/index",
      name: "operation",
      component: () => import("@/views/operation/index.vue"),
      meta: {
        title: "operation"
      }
    },
    {
      path: "/operation/components/modify",
      name: "modify",
      component: () => import("@/views/operation/components/modify.vue"),
      meta: {
        title: "modify",
        showLink: false
      }
    }
  ]
} satisfies RouteConfigsTable;
