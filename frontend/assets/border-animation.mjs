document.addEventListener("bee:board-loaded", event => {
  console.log("loaded!", event)
  const els = Array.from(event.target.querySelectorAll(".letter"))
    .map(el => {
      const rect = el.getBoundingClientRect()
      const centerX = rect.x + (rect.width / 2)
      const centerY = rect.y + (rect.height / 2)
      return {
        el,
        rect,
        center: [centerX, centerY]
      }
    })

  const [maxX, maxY] = els.reduce(([mx, my], el) => {
    return [
      Math.max(mx, el.center[0]),
      Math.max(my, el.center[1]),
    ]
  }, [-Infinity, -Infinity])

  const [minX, minY] = els.reduce(([mx, my], el) => {
    return [
      Math.min(mx, el.center[0]),
      Math.min(my, el.center[1]),
    ]
  }, [Infinity, Infinity])

  event.target
    .addEventListener("mousemove", event => {    
      const [eventX, eventY] = [
        Math.min(maxX, Math.max(minX, event.x)),
        Math.min(maxY, Math.max(minY, event.y)),
      ]
      
      els.forEach(el => {
        const rect = el.rect
        const [x, y] = [eventX - rect.x, eventY - rect.y]

        el.el.style.setProperty("--border-gradient-x", `${x}px`)
        el.el.style.setProperty("--border-gradient-y", `${y}px`)
      })
    })
}, {
  once: true
})
