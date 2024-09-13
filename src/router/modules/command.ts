export default {
  path: "/command",
  redirect: "/command/cmd",
  meta: {
    icon: "command",
    title: "command",
    rank: 1
  },
  children: [
    {
      path: "/command/cmd",
      name: "command",
      component: () => import("@/views/command/cmd.vue"),
      meta: {
        title: "command"
      }
    },
    {
      path: "/command/cat",
      name: "cat",
      component: () => import("@/views/command/cat.vue"),
      meta: {
        title: "cat",
        showLink: false
      }
    }
  ]
} satisfies RouteConfigsTable;
