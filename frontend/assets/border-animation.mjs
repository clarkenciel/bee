document.addEventListener("bee:board-loaded", event => {
  const els = Array.from(event.target.querySelectorAll(".letter"))
    .map(el => {
      const rect = el.getBoundingClientRect()
      const centerX = rect.x + (rect.width / 2)
      const centerY = rect.y + (rect.height / 2)
      return {
        el,
        rect,
        center: { x: centerX, y: centerY },
        distanceFrom(point) {
          return distance(this.center, point)
        }
      }
    })

  function distance({ x: x1, y: y1 }, { x: x2, y: y2 }) {
    return Math.sqrt(Math.pow(Math.abs(x2 - x1), 2) + Math.pow(Math.abs(y2 - y1), 2))
  }

  function nearest(target) {
    return els.reduce((out, el) => el.distanceFrom(target) < out.distanceFrom(target) ? el : out)
  }

  function ease(n) {
    return 1 - Math.pow(1 - n, 3)
  }

  let currentLitPos = {x: 0, y: 0}
  let cancelCurrentAnimation = null
  function chase(target, start) {
    const animationDurationMillis = 250
    const startTime = performance.now()
    const endTime = startTime + animationDurationMillis
    const totalXDist = target.x - start.x
    const totalYDist = target.y - start.y
    const animation = requestAnimationFrame(function animate(now) {
      const elapsed = (now - startTime) / animationDurationMillis
      const progressed = ease(elapsed)
      const xDist = progressed * totalXDist
      const yDist = progressed * totalYDist
      currentLitPos = {
        x: xDist + start.x,
        y: yDist + start.y,
      }

      moveLightTo(currentLitPos)
      if (distance(currentLitPos, target) < 0.01) {
        cancelCurrentAnimation()
        return
      }

      const animation = requestAnimationFrame(animate)
      cancelCurrentAnimation = function () { cancelAnimationFrame(animation) }
    })
    cancelCurrentAnimation = function () { cancelAnimationFrame(animation) }
  }

  function moveLightTo(target) {
    console.log("moving light to", target)
    els.forEach(el => {
      const rect = el.rect
      const [x, y] = [target.x - rect.x, target.y - rect.y]

      el.el.style.setProperty("--border-gradient-x", `${x}px`)
      el.el.style.setProperty("--border-gradient-y", `${y}px`)
    })
  }

  let currentTarget = null
  event.target.addEventListener("mousemove", event => {    
    if (currentTarget === null) {
      currentTarget = nearest(event)
      currentLitPos = currentTarget.center
      moveLightTo(currentTarget.center)
      return
    }

    const newTarget = nearest(event)
    if (newTarget.el === currentTarget.el) return

    console.log("chasing", newTarget.center, "from", currentLitPos)
    cancelCurrentAnimation && cancelCurrentAnimation()
    chase(newTarget.center, currentLitPos)
    currentTarget = newTarget
  })
}, {
  once: true
})
