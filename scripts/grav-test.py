import numpy as np
import matplotlib.pyplot as plt

plt.style.use("root")

p = 3
H = 10
A = 1

def dist(x):
    r = np.sqrt(x[0]**2 + x[1]**2 + x[2]**2)
    P = (np.abs(x[0])**p + np.abs(x[1])**p + np.abs(x[2])**p)**(1.0 / p)
    return r * (1.0 - (P / r - 1.0) * (H / r))

def grav(x, v):
    R = np.linalg.norm(x)
    e = x / R
    ae = np.abs(e)
    Q = np.sum(ae**p)**(1.0 / p)
    gradPe = np.sign(e) * ae**(p - 1) * Q**(1 - p)
    u = Q - 1.0

    s = R + u * H
    fv = R / s

    gamma = -(H / s**2) * (gradPe @ v - (2.0 * Q - 1.0) * (e @ v))
    vel = (v - s * gamma * e) / fv

    a = e @ vel
    b = vel @ vel
    sig = gradPe @ vel
    Phi = ((p - 1) / Q * (Q**(2 - p) * np.sum(ae**(p - 2) * vel**2) - sig**2)
           - 4.0 * sig * a - Q * b + 5.0 * Q * a**2
           + u * (3.0 * a**2 - b))

    return -A * e / s**2 + (2.0 * gamma * vel - H * Phi / s**2 * e)

def run(v0, x0=2):
    dt = 0.1
    x = np.array([x0, 0., 0])
    v = np.array([0., v0, 0])
    xs = []
    for _ in range(30000):
        v += grav(x,v)*dt
        x += v*dt
        xs.append(np.copy(x))
    return np.array(xs)

def main():
    line = np.linspace(-30, 30, 100)
    a, b = np.meshgrid(line, line)
    dists = dist([a, b, 0])

    fig, ax = plt.subplots()
    xs = run(1.1/np.sqrt(10), 10)
    ax.plot(xs[:,0], xs[:,1])
    xs = run(1/np.sqrt(20), 20)
    ax.plot(xs[:,0], xs[:,1])
    xs = run(1/np.sqrt(30), 30)
    ax.plot(xs[:,0], xs[:,1])
    ax.set_xlim(-30,30)
    ax.set_ylim(-30,30)
    ax.set_aspect("equal")
    ax.contour(line, line, dists, linewidths=1, colors='k', levels=[0.5, 1, 2, 3])
    fig.savefig("dbg-figs/coords.png")

if __name__ == "__main__":
    main()