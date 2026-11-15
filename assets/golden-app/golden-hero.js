/* Golden live 3D homepage. Frontend only; no API keys. */
(() => {
  const $ = s => document.querySelector(s);
  const host = $('#golden-3d'), hero = $('#hero');
  if (!host || !window.THREE) return;
  const DEG = Math.PI / 180;
  const state = { lat: 40.758, lon: -73.9855, name: 'New York · approximate', tz: 'America/New_York', temp: null, cloud: 25, wind: 5, code: 0, sunrise: '', sunset: '', source: 'fallback', buildings: false };
  const scene = new THREE.Scene();
  scene.background = new THREE.Color('#1b4572');
  scene.fog = new THREE.FogExp2('#75859c', .0016);
  const camera = new THREE.PerspectiveCamera(59, 1, 1, 6500);
  camera.position.set(180, 240, 510);
  const renderer = new THREE.WebGLRenderer({antialias: true, powerPreference: 'high-performance'});
  renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 1.6));
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.outputEncoding = THREE.sRGBEncoding;
  host.appendChild(renderer.domElement);
  const world = new THREE.Group(); scene.add(world);
  const ground = new THREE.Mesh(new THREE.PlaneGeometry(5000,5000), new THREE.MeshStandardMaterial({color:'#2c4560',roughness:1}));
  ground.rotation.x = -Math.PI/2; ground.receiveShadow = true; scene.add(ground);
  const ambient = new THREE.HemisphereLight(0xb5cfff,0x222536,1); scene.add(ambient);
  const sunlight = new THREE.DirectionalLight(0xffd0a0,2.1);
  sunlight.castShadow = true; sunlight.shadow.mapSize.set(1024,1024);
  sunlight.shadow.camera.left = sunlight.shadow.camera.bottom = -800;
  sunlight.shadow.camera.right = sunlight.shadow.camera.top = 800;
  sunlight.shadow.camera.near = 1; sunlight.shadow.camera.far = 3500;
  sunlight.shadow.bias = -.0003; scene.add(sunlight, sunlight.target);
  const sunDisc = new THREE.Mesh(new THREE.SphereGeometry(33,24,16), new THREE.MeshBasicMaterial({color:0xffe7b3})); scene.add(sunDisc);
  const cloudGroup = new THREE.Group(); scene.add(cloudGroup);
  const cloudMat = new THREE.MeshLambertMaterial({color:0xe2d9d2,transparent:true,opacity:.5,depthWrite:false});
  for(let i=0;i<18;i++){
    const g=new THREE.Group();
    for(let j=0;j<5;j++){const m=new THREE.Mesh(new THREE.SphereGeometry(1,10,8),cloudMat);m.scale.set(75+Math.random()*55,12+Math.random()*15,26+Math.random()*20);m.position.set(j*44,Math.random()*10,Math.random()*35);g.add(m)}
    g.position.set((Math.random()-.5)*2300,400+Math.random()*200,(Math.random()-.5)*2000);cloudGroup.add(g);
  }
  const rainCount=650, rainPos=new Float32Array(rainCount*3);
  for(let i=0;i<rainCount;i++){rainPos[i*3]=(Math.random()-.5)*900;rainPos[i*3+1]=Math.random()*550;rainPos[i*3+2]=(Math.random()-.5)*900}
  const rainGeo=new THREE.BufferGeometry();rainGeo.setAttribute('position',new THREE.BufferAttribute(rainPos,3));
  const rain=new THREE.Points(rainGeo,new THREE.PointsMaterial({color:0xc9d7f1,size:2.5,transparent:true,opacity:.65}));scene.add(rain);
  const buildingMat=new THREE.MeshStandardMaterial({color:0x8a91a0,roughness:.84,metalness:.08});
  const glassMat=new THREE.MeshStandardMaterial({color:0x56697d,roughness:.3,metalness:.28});
  const meshes=[];
  function clearBuildings(){meshes.forEach(m=>{world.remove(m);m.geometry.dispose()});meshes.length=0}
  function addBlock(x,z,w,d,h,glass=false){
    if(h<3)return;
    const mesh=new THREE.Mesh(new THREE.BoxGeometry(Math.max(2,w),h,Math.max(2,d)),glass?glassMat:buildingMat);
    mesh.position.set(x,h/2,z);mesh.castShadow=true;mesh.receiveShadow=true;world.add(mesh);meshes.push(mesh);
  }
  function placeholder(){clearBuildings();let seed=91;const rand=()=>((seed=(seed*1664525+1013904223)>>>0)/4294967296);
    for(let x=-900;x<900;x+=65)for(let z=-900;z<900;z+=65){if(rand()<.32)continue;addBlock(x,z,25+rand()*32,25+rand()*32,20+Math.pow(rand(),3)*350,rand()>.65)}
    state.buildings=false; updateHUD();
  }
  function sunPosition(time,lat,lon){
    const d=new Date(time),day=(Date.UTC(d.getUTCFullYear(),d.getUTCMonth(),d.getUTCDate())-Date.UTC(d.getUTCFullYear(),0,0))/86400000+1;
    const gamma=2*Math.PI/365*(day-1+(d.getUTCHours()-12)/24);
    const eq=229.18*(.000075+.001868*Math.cos(gamma)-.032077*Math.sin(gamma)-.014615*Math.cos(2*gamma)-.040849*Math.sin(2*gamma));
    const dec=.006918-.399912*Math.cos(gamma)+.070257*Math.sin(gamma)-.006758*Math.cos(2*gamma)+.000907*Math.sin(2*gamma)-.002697*Math.cos(3*gamma)+.00148*Math.sin(3*gamma);
    const mins=d.getUTCHours()*60+d.getUTCMinutes()+d.getUTCSeconds()/60;
    const ha=(mins+eq+4*lon)/4*DEG-Math.PI;
    const p=lat*DEG, el=Math.asin(Math.sin(p)*Math.sin(dec)+Math.cos(p)*Math.cos(dec)*Math.cos(ha));
    const az=(Math.atan2(Math.sin(ha),Math.cos(ha)*Math.sin(p)-Math.tan(dec)*Math.cos(p))/DEG+540)%360;
    return {az,el:el/DEG};
  }
  function resize(){const w=host.clientWidth,h=host.clientHeight;camera.aspect=w/Math.max(h,1);camera.updateProjectionMatrix();renderer.setSize(w,h,false)}
  addEventListener('resize',resize);resize();
  const codeName=c=>c===0?'Clear sky':c<=2?'Partly cloudy':c===3?'Overcast':c<=48?'Fog':c<=67?'Rain':c<=77?'Snow':c<=82?'Showers':'Storm';
  function updateHUD(){
    $('#hplace').textContent=state.name;
    $('#htime').textContent=new Date().toLocaleTimeString('en-US',{timeZone:state.tz,hour:'numeric',minute:'2-digit'});
    $('#hwx').textContent=state.temp==null?'Loading live weather…':`${Math.round(state.temp)}°C · ${codeName(state.code)} · ${Math.round(state.wind)} km/h wind`;
    $('#hsun').textContent=[state.sunrise&&`Sunrise ${state.sunrise}`,state.sunset&&`Sunset ${state.sunset}`].filter(Boolean).join('  ·  ');
    $('#hnote').textContent=(state.buildings?state.attribution:'Preview geometry only. Waiting for real height data')+' · Weather: Open-Meteo';
  }
  async function weather(){try{
    const u=`https://api.open-meteo.com/v1/forecast?latitude=${state.lat}&longitude=${state.lon}&current=temperature_2m,weather_code,cloud_cover,wind_speed_10m&daily=sunrise,sunset&timezone=auto&forecast_days=1`;
    const r=await fetch(u);if(!r.ok)throw Error(r.status);const j=await r.json();
    Object.assign(state,{temp:j.current.temperature_2m,cloud:j.current.cloud_cover,wind:j.current.wind_speed_10m,code:j.current.weather_code,tz:j.timezone,sunrise:j.daily.sunrise[0].slice(11),sunset:j.daily.sunset[0].slice(11)});updateHUD();
  }catch(e){$('#hwx').textContent='Weather temporarily unavailable'} }
  /* Buildings from the golden server's height map (/api/scene: Vexcel, or USGS LiDAR outside Vexcel's area).
     Ring 1 has 4 m cells over ~1 km; every cell standing 3 m or more above the local ground becomes a box. */
  // the page's API base (index.html CFG: location.origin, or origin+?api=/golden on burvy.dev)
  const API = typeof CFG==='object' && CFG.base ? CFG.base : /^https?:/.test(location.protocol) ? location.origin : 'http://localhost:8787';
  async function buildings(){
    try{
      const c=new AbortController(),to=setTimeout(()=>c.abort(),120000);
      const get=async u=>{const r=await fetch(API+u,{signal:c.signal});if(!r.ok)throw Error(r.status);return r};
      const meta=await (await get(`/api/scene?lat=${state.lat}&lon=${state.lon}&radius_m=500&detail=phone`)).json();
      const buf=new DataView(await (await get(meta.data_url)).arrayBuffer());clearTimeout(to);
      const ring=meta.rings.find(r=>r.cell_m>=2)||meta.rings[meta.rings.length-1],n=ring.width,cell=ring.cell_m;
      const hs=new Float32Array(n*n);
      for(let i=0;i<n*n;i++){const v=buf.getUint16(ring.byte_offset+i*2,true);hs[i]=v===meta.nodata?NaN:v/10}
      // local ground: the 15th percentile of the ring, so hills don't read as buildings
      const sorted=Array.from(hs).filter(v=>!isNaN(v)).sort((x,y)=>x-y);if(sorted.length<100)throw Error('no heights');
      const ground=sorted[Math.floor(sorted.length*.15)],tall=[];
      for(let r=0;r<n;r++)for(let k=0;k<n;k++){const h=hs[r*n+k]-ground;if(h>=3)tall.push([(k-n/2+.5)*cell,(r-n/2+.5)*cell,Math.min(550,h)])}
      if(tall.length<20)throw Error('too few buildings');
      clearBuildings();
      const geo=new THREE.BoxGeometry(cell,1,cell);geo.translate(0,.5,0);
      const mesh=new THREE.InstancedMesh(geo,buildingMat,tall.length),m=new THREE.Matrix4();
      tall.forEach(([x,z,h],i)=>{m.makeScale(1,h,1);m.setPosition(x,0,z);mesh.setMatrixAt(i,m)});
      mesh.castShadow=true;mesh.receiveShadow=true;world.add(mesh);meshes.push(mesh);
      state.attribution=meta.attribution;state.buildings=true;updateHUD();
    }catch(e){if(!meshes.length)placeholder();state.buildings=false;updateHUD()}
  }
  async function setLocation(loc){Object.assign(state,loc);state.temp=null;updateHUD();placeholder();await Promise.all([weather(),buildings()]);}
  async function locate(){
    const geo=new Promise(resolve=>{if(!navigator.geolocation)return resolve(null);navigator.geolocation.getCurrentPosition(p=>resolve({lat:p.coords.latitude,lon:p.coords.longitude,source:'gps'}),()=>resolve(null),{timeout:6500,maximumAge:600000})});
    let loc=await geo;
    if(!loc){try{const r=await fetch('https://ipapi.co/json/');const j=await r.json();if(j.latitude&&j.longitude)loc={lat:j.latitude,lon:j.longitude,name:`${j.city||'Your city'} · approximate`,source:'ip'}}catch(e){}}
    if(!loc)loc={lat:40.758,lon:-73.9855,name:'New York · sample location',source:'fallback'};
    if(loc.source==='gps'){try{const r=await fetch(`https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${loc.lat}&longitude=${loc.lon}&localityLanguage=en`);const j=await r.json();loc.name=j.city||j.locality||'Your location'}catch(e){loc.name='Your location'}}
    await setLocation(loc);
  }
  let closed=false,frame=0;
  function animate(){if(closed)return;requestAnimationFrame(animate);frame++;
    const p=sunPosition(Date.now(),state.lat,state.lon),a=p.az*DEG,e=p.el*DEG;
    const dir=new THREE.Vector3(Math.sin(a)*Math.cos(e),Math.sin(e),-Math.cos(a)*Math.cos(e));
    sunlight.position.copy(dir).multiplyScalar(1300);sunDisc.position.copy(dir).multiplyScalar(2400);
    sunDisc.visible=p.el>-4;
    const daylight=Math.max(0,Math.min(1,(p.el+6)/20));
    sunlight.intensity=.04+daylight*2.6*(1-state.cloud/170);
    ambient.intensity=.55+daylight*.6;
    // sky-blue theme (task 032): twilight blue at night, sunset orange at dusk, sky blue by day
    const night=new THREE.Color('#1b4572'),dusk=new THREE.Color('#e8875a'),day=new THREE.Color('#7cc2f0');
    const sky=p.el<0?night.clone().lerp(dusk,Math.max(0,(p.el+12)/12)):dusk.clone().lerp(day,Math.min(1,p.el/24));
    scene.background.copy(sky);scene.fog.color.copy(sky);
    cloudMat.opacity=Math.min(.85,.12+state.cloud/120);
    cloudGroup.position.x+=.03+state.wind*.006;
    if(cloudGroup.position.x>450)cloudGroup.position.x=-450;
    rain.visible=(state.code>=51&&state.code<=67)||(state.code>=80&&state.code<=82)||(state.code>=95);
    if(rain.visible){const arr=rainGeo.attributes.position;for(let i=0;i<rainCount;i++){let y=arr.getY(i)-4;if(y<0)y=550;arr.setY(i,y)}arr.needsUpdate=true}
    const t=performance.now()*.00006;
    camera.position.set(Math.sin(t)*440,230+Math.sin(t*.7)*25,Math.cos(t)*440);
    camera.lookAt(0,95,0);
    renderer.render(scene,camera);
  }
  function close(){closed=true;hero.classList.add('out');setTimeout(()=>{hero.remove();renderer.dispose()},1150)}
  $('#hgo').addEventListener('click',close);
  document.addEventListener('keydown',e=>{if(e.key==='Escape')close()});
  placeholder();updateHUD();animate();locate();
  setInterval(()=>{if(!closed)updateHUD()},1000);
  setInterval(()=>{if(!closed)weather()},600000);
  setInterval(()=>{if(!closed)buildings()},3600000);
})();
