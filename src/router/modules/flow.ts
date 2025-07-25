export default {
  path: "/flow",
  redirect: "/flow/flow",
  meta: {
    icon: "flowChart",
    title: "flow",
    rank: 2
  },
  children: [
    {
      path: "/flow/flow",
      name: "flow",
      component: () => import("@/views/flow/flow.vue"),
      meta: {
        title: "flow"
      }
    }
  ]
} satisfies RouteConfigsTable;
