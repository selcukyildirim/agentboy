# AI Workforce --- Ürün Özellikleri v2

**Doküman Türü:** Ürün Özellikleri / Product Feature Specification\
**Dil:** Türkçe\
**Ürün:** AI Workforce\
**Sürümler:** Free Desktop + Company Server / Paid

------------------------------------------------------------------------

# 1. Ürün Vizyonu

AI Workforce, çalışanların günlük operasyonel işlerini yapabilen ve
şirket sistemleri üzerinde kontrollü şekilde çalışabilen yapay zekâ
ajanlarından oluşan bir iş platformudur.

Ürün iki temel katmandan oluşur:

## Free Desktop

Kullanıcının bilgisayarında çalışan kişisel AI worker.

Temel amacı:

> Kullanıcının dosyalarla yaptığı tekrar eden işleri hızlandırmak,
> dokümanlarını anlamak, analiz yapmak ve karar desteği sağlamak.

## Company Server / Paid

Şirket sistemlerine bağlanan merkezi AI workforce platformu.

Temel amacı:

> Şirket operasyonlarını izlemek, analiz etmek, karar önermek ve şirket
> politikaları dahilinde aksiyon almak.

Temel ürün ayrımı:

> **Free = Benim bilgisayarımda benim için çalışır.**\
> **Paid = Şirketimin sistemlerinde şirketim adına çalışır.**

------------------------------------------------------------------------

# 2. Temel Kullanıcı Deneyimi

Ürün klasik boş chatbot ekranıyla başlamamalıdır.

İlk açılışta kullanıcıya:

**"Hangi departmanda çalışıyorsun?"**

sorulur.

Seçenekler:

-   Finans
-   Muhasebe
-   Satış
-   Satın Alma
-   İnsan Kaynakları
-   Operasyon
-   Lojistik / Depo
-   Yönetim
-   PMO / Proje Yönetimi
-   Hukuk / Sözleşme

Departman seçildikten sonra kullanıcıya hazır AI çalışanları gösterilir.

Örnek:

``` text
Satın Alma

[ Tedarikçileri Karşılaştır ]
[ Fiyat Geçmişini Analiz Et ]
[ Satın Alma Siparişlerini İncele ]
[ Tedarikçi Performansını Analiz Et ]
[ RFQ Hazırla ]
[ Satın Alma Kararı Ver ]

--------------------------------

Ne yapmak istediğini yaz...
```

Kullanıcı ister hazır görevi seçebilir ister doğal dil ile doğrudan
talimat verebilir.

------------------------------------------------------------------------

# 3. Free Desktop Ana Özellikleri

## 3.1 Doğal Dil ile İş Yaptırma

Kullanıcı örneğin:

> Bu klasördeki satış Excel'lerini birleştir, mükerrer siparişleri
> temizle ve müşteri bazında satış raporu oluştur.

yazabilir.

Sistem:

1.  Talebi anlar.
2.  Gerekli dosyaları belirler.
3.  Kullanıcıdan gerekli klasör/dosya erişimini alır.
4.  İş planını oluşturur.
5.  Deterministik işlemleri lokal araçlarla yapar.
6.  Gerekiyorsa AI modelinden destek alır.
7.  Sonucu kullanıcıya gösterir.
8.  İstenirse çıktı dosyası üretir.

------------------------------------------------------------------------

# 4. Dosya ve Veri İşleme

Free Desktop aşağıdaki temel işlemleri destekler.

## Excel / CSV

-   Dosya birleştirme
-   Dosya bölme
-   Kolon eşleme
-   Veri temizleme
-   Duplicate temizleme
-   Filtreleme
-   Gruplama
-   Pivot/özet oluşturma
-   Formül ve hesaplama
-   Dosya karşılaştırma
-   Mutabakat
-   Veri doğrulama
-   Anomali tespiti
-   XLSX ↔ CSV dönüşümü
-   Rapor oluşturma

## PDF / DOCX / TXT

-   Doküman okuma
-   Özetleme
-   Bilgi çıkarma
-   Alan çıkarma
-   Doküman karşılaştırma
-   Madde/başlık bulma
-   Tablo çıkarma
-   Soru-cevap
-   Kaynak göstererek analiz

## JSON / XML

-   Okuma
-   Dönüştürme
-   Mapping
-   Validation
-   Flatten
-   Alan seçme
-   Veri karşılaştırma

## Dosya İşlemleri

-   Yeniden adlandırma
-   Sınıflandırma
-   Klasörleme
-   Taşıma
-   Kopyalama
-   Duplicate bulma
-   Dosya türüne göre organize etme

------------------------------------------------------------------------

# 5. Kendi AI Modelini Kullan --- BYOK

Kullanıcı kendi API anahtarını tanımlayabilir.

Desteklenecek sağlayıcılar:

-   OpenAI
-   Anthropic Claude
-   Google Gemini
-   OpenAI-compatible servisler
-   Ollama / lokal modeller

Ayar ekranında:

``` text
AI Providers

OpenAI
● Bağlı
Model: [ Kullanılabilir modeller ]
[Test Et]

Anthropic Claude
○ Bağlı değil
[API Key Gir]

Google Gemini
○ Bağlı değil
[API Key Gir]

Local / Ollama
● Çalışıyor
Model: [...]
```

Kullanıcı hangi modelin kullanılacağını seçebilir.

------------------------------------------------------------------------

# 6. Akıllı Model Seçimi

Üç kullanım modu bulunur.

## Manuel

Kullanıcı modeli kendisi seçer.

## Smart

Sistem göreve göre uygun modeli seçer.

Örneğin:

``` text
Basit sınıflandırma
→ hızlı/düşük maliyetli model

Karmaşık analiz
→ güçlü reasoning modeli

Görsel doküman
→ vision destekli model

Embedding
→ embedding modeli
```

## Privacy First

Öncelik sırası:

``` text
Lokal kod
→ Lokal model
→ Maskelenmiş cloud context
→ Minimum gerekli cloud context
```

------------------------------------------------------------------------

# 7. Veri Gizliliği

Free Desktop'ın temel ürün vaadi:

> Şirket verisini mümkün olduğunca cihazda tut.

Her iş için sistem verinin dışarı çıkıp çıkmadığını belirler.

## Veri Seviyeleri

### L0 --- Local Only

Hiçbir business data cihazdan çıkmaz.

### L1 --- Metadata

Sadece:

-   kolon isimleri
-   veri tipleri
-   dosya yapısı
-   satır sayısı

gibi metadata gönderilebilir.

### L2 --- Maskelenmiş Veri

Örneğin:

``` text
Ahmet Yılmaz → PERSON_001
ABC Holding → COMPANY_001
125.000 TL → AMOUNT_001
```

### L3 --- Minimum Gerekli Veri

Sadece AI'ın görevi tamamlaması için gereken alanlar gönderilir.

### L4 --- Tam İçerik

Varsayılan değildir.

Kullanıcı/policy açık şekilde izin vermelidir.

------------------------------------------------------------------------

# 8. AI Veri Çıkış Ekranı

Kullanıcı işlem sonunda görebilir:

``` text
İşlem Özeti

✓ 4 Excel dosyası lokal işlendi
✓ 18.420 satır cihazda analiz edildi
✓ Dosyalar cloud'a yüklenmedi

AI kullanımı:
Claude

Gönderilen:
- 12 anonimleştirilmiş kayıt
- kolon isimleri

Kişisel veri:
Gönderilmedi
```

Bu özellik ürünün güven unsurunun önemli parçalarından biridir.

------------------------------------------------------------------------

# 9. Personal Knowledge Workspace

Kullanıcı kendi dokümanlarını ürüne ekleyebilir.

Örneğin:

``` text
Bilgi Alanım

├── Şirket Prosedürleri
├── Sözleşmeler
├── Finans
├── Satın Alma
├── Projeler
└── Kişisel Çalışmalar
```

Desteklenen içerikler:

-   PDF
-   DOCX
-   XLSX
-   CSV
-   TXT
-   Markdown
-   JSON
-   XML

Dokümanlar mümkün olduğunca lokal indexlenir.

------------------------------------------------------------------------

# 10. Gelişmiş RAG

Ürün yalnızca:

> "PDF yükle ve soru sor."

deneyimi sunmaz.

Amaç:

> Doküman + yapılandırılmış veri + şirket kuralı + geçmiş bilgi
> kullanarak analiz ve karar desteği üretmek.

Örnek:

Kullanıcı yükler:

-   Satın Alma Prosedürü.pdf
-   SupplierContract.pdf
-   Teklifler.xlsx
-   GeçmişAlımlar.xlsx

Sonra sorar:

> Bu tedarikçiden satın almalıyım?

Sistem:

``` text
Teklif
+
Geçmiş fiyatlar
+
Sözleşme
+
Satın alma prosedürü
+
Eksik bilgiler
        ↓
Karar Analizi
```

------------------------------------------------------------------------

# 11. Kanıtlı Cevaplar

Karar/analiz cevapları mümkün olduğunda aşağıdaki formatta sunulur:

``` text
Öneri
Revizyon iste.

Güven
%91

Neden
Fiyat son 6 aylık ortalamadan %17,4 yüksek.

Şirket Kuralı
%10 üzerindeki fiyat artışında ikinci teklif gerekiyor.

Kanıtlar
Satın Alma Prosedürü — Madde 4.2
SupplierContract.pdf — Madde 7
GeçmişAlımlar.xlsx — ilgili kayıtlar

Eksik Bilgi
İkinci teklif bulunamadı.

Gerekli Aksiyon
İkinci teklif talep et.
```

Agent yeterli veri yoksa bunu açıkça belirtir.

------------------------------------------------------------------------

# 12. Karar Hafızası

Sistem kullanıcının geçmiş kararlarını lokal olarak saklayabilir.

Örnek:

``` text
12 Haziran
Supplier ABC
Fiyat artışı: %14
Karar: Reddedildi
Neden: Şirket limitinin üzerinde.

3 Temmuz
Supplier XYZ
Fiyat artışı: %12
Karar: Onaylandı
Neden: Kritik stok tükenme riski.
```

Daha sonraki analizlerde geçmiş kararlar yardımcı bağlam olarak
kullanılabilir.

Geçmiş kararlar otomatik şirket kuralı kabul edilmez.

------------------------------------------------------------------------

# 13. Workflow Kaydetme

Kullanıcı yaptığı işi workflow olarak kaydedebilir.

Örnek:

> Her hafta yaptığım bu işi kaydet.

Workflow:

``` text
Sales/*.xlsx oku
→ Dosyaları birleştir
→ Duplicate OrderId temizle
→ Dealer bazında grupla
→ Revenue hesapla
→ Excel oluştur
→ Yönetici özeti oluştur
→ Reports/Weekly klasörüne kaydet
```

Sonraki hafta:

> Haftalık satış raporunu hazırla.

demesi yeterlidir.

Free sürümde workflow kullanıcı tarafından manuel başlatılır.

------------------------------------------------------------------------

# 14. Free Departman Ajanları

## 14.1 Finans

### Cash Flow Analyst

Nakit giriş/çıkışlarını analiz eder ve dönemsel nakit görünümü
oluşturur.

### Collection Analyst

Gecikmiş alacakları analiz eder ve tahsilat önceliği çıkarır.

### Bank Reconciliation Agent

Banka hareketleri ile cari/muhasebe kayıtlarını eşleştirir.

### Budget Variance Agent

Bütçe ve gerçekleşeni karşılaştırır.

### Financial Report Agent

Finans dosyalarından yönetim raporu üretir.

### Expense Analyst

Masrafları analiz eder, sıra dışı giderleri ve olası tekrarları bulur.

------------------------------------------------------------------------

## 14.2 Muhasebe

### Invoice Reader

Faturalardan yapılandırılmış veri çıkarır.

### Invoice Control Agent

Fatura tutarsızlıklarını ve olası duplicate kayıtları bulur.

### Account Reconciliation Agent

Cari ekstreleri karşılaştırır.

### Closing Assistant

Ay sonu kapanış kontrolüne yardımcı olur.

### Ledger Analysis Agent

Muhasebe hareketlerindeki sıra dışı kayıtları bulur.

### Accounting Document Organizer

Muhasebe dokümanlarını sınıflandırır ve organize eder.

------------------------------------------------------------------------

## 14.3 Satış

### Sales Performance Analyst

Müşteri, ürün, bölge ve satışçı performansını analiz eder.

### Quotation Analyst

Teklifleri ve teklif revizyonlarını karşılaştırır.

### Customer Analysis Agent

Müşteri satın alma davranışını analiz eder.

### Lost Customer Finder

Satın alma hacmi düşen veya sipariş vermeyi bırakan müşterileri bulur.

### Margin Analysis Agent

Ürün/müşteri bazında marj analizi yapar.

### Sales Report Agent

Satış yönetim raporu oluşturur.

------------------------------------------------------------------------

## 14.4 Satın Alma

### Supplier Comparison Agent

Tedarikçi tekliflerini normalize edip karşılaştırır.

### Price History Agent

Geçmiş satın alma fiyatlarını analiz eder.

### Purchase Order Analyst

Siparişlerde sıra dışı fiyat, miktar veya tedarikçi kullanımını bulur.

### Supplier Performance Agent

Fiyat, termin ve performans verilerini analiz eder.

### RFQ Assistant

RFQ hazırlamaya ve teklifleri karşılaştırmaya yardımcı olur.

### Procurement Decision Agent

Teklif, geçmiş alım, sözleşme ve prosedürleri birlikte değerlendirerek
karar desteği sağlar.

------------------------------------------------------------------------

## 14.5 İnsan Kaynakları

### CV Analysis Agent

CV bilgilerini yapılandırır ve pozisyon kriterleriyle karşılaştırır.

### Employee Document Agent

Personel evraklarını organize eder ve eksikleri bulur.

### Leave Analysis Agent

İzin verilerini analiz eder.

### Attendance Analyst

Puantaj/devam verilerindeki sıra dışı durumları bulur.

### HR Report Agent

İK raporları oluşturur.

### HR Policy Assistant

Şirket İK prosedürlerinden kaynaklı cevap üretir.

------------------------------------------------------------------------

## 14.6 Operasyon

### Operations Analyst

Operasyon verilerini analiz eder.

### Exception Finder

Normal akıştan sapan kayıtları bulur.

### SLA Monitor Agent

SLA aşım ve risklerini analiz eder.

### Process Bottleneck Agent

Süreç darboğazlarını belirler.

### Quality Analysis Agent

Hata/iade/şikâyet verilerini analiz eder.

### Daily Operations Brief Agent

Günlük operasyon yönetici özeti oluşturur.

------------------------------------------------------------------------

## 14.7 Lojistik / Depo

### Inventory Analysis Agent

Stok durumunu analiz eder.

### Slow Moving Agent

Yavaş hareket eden stokları bulur.

### Stockout Risk Agent

Stok tükenme risklerini analiz eder.

### Shipment Analysis Agent

Sevkiyat ve taşıyıcı performansını analiz eder.

### Inventory Reconciliation Agent

Sayım ve sistem stoklarını karşılaştırır.

### Warehouse Report Agent

Depo operasyon raporu oluşturur.

------------------------------------------------------------------------

## 14.8 Yönetim

### Executive Brief Agent

Farklı raporlardan kısa yönetici özeti çıkarır.

### KPI Analyst

KPI değişimlerini analiz eder.

### Anomaly Agent

Finans, satış ve operasyon verilerindeki önemli anomalileri bulur.

### Decision Assistant

Doküman ve veriler üzerinden yönetim karar desteği sağlar.

### Meeting Prep Agent

Toplantı öncesi briefing hazırlar.

### Management Report Agent

Yönetim raporu oluşturur.

------------------------------------------------------------------------

## 14.9 PMO / Proje Yönetimi

### Project Status Agent

Proje durumunu analiz eder.

### Risk Agent

Proje risklerini belirler.

### Action Tracker

Toplantı notlarından aksiyon, sorumlu ve tarih çıkarır.

### Project Cost Analyst

Proje bütçesi ve gerçekleşeni karşılaştırır.

### Timeline Analyzer

Planlanan ve gerçekleşen tarihleri analiz eder.

### Project Brief Agent

Yönetim için proje özeti oluşturur.

------------------------------------------------------------------------

## 14.10 Hukuk / Sözleşme

### Contract Reader

Sözleşmenin temel şartlarını çıkarır.

### Contract Comparison Agent

Sözleşme/revizyon farklarını gösterir.

### Obligation Extractor

Yükümlülük, tarih ve ödeme şartlarını çıkarır.

### Renewal Tracker

Sözleşme bitiş/yenileme tarihlerini belirler.

### Clause Finder

Belirli konuyla ilgili sözleşme maddelerini bulur.

### Contract Policy Checker

Sözleşmeyi şirket playbook/prosedürü ile karşılaştırıp inceleme
noktalarını işaretler.

------------------------------------------------------------------------

# 15. İlk Free Release

Ürün ilk sürümde 60 yüzeysel agent ile çıkmamalıdır.

İlk sürümde 9 güçlü agent hedeflenir.

## Finans

1.  Bank Reconciliation Agent
2.  Budget Variance Agent
3.  Expense Analyst

## Muhasebe

4.  Invoice Reader
5.  Invoice Control Agent
6.  Account Reconciliation Agent

## Satın Alma

7.  Supplier Comparison Agent
8.  Price History Agent
9.  Procurement Decision Agent

Bu agent'lar ortak skill altyapısını doğruladıktan sonra katalog 60
agent'a genişletilir.

------------------------------------------------------------------------

# 16. Agent Sonuç Ekranı

Her agent'ın sonucu mümkün olduğunca standart bir yapı kullanmalıdır.

``` text
İŞLEM TAMAMLANDI

Özet
3 dosya analiz edildi.
18.420 kayıt işlendi.

Bulgular
• 27 gecikmiş alacak
• 8 olası eşleşmeyen banka hareketi
• 4 bakiye farkı

Önerilen Aksiyonlar
[ Mutabakat Raporu Oluştur ]
[ Excel Olarak Kaydet ]
[ Detayları Göster ]

AI Kullanımı
Claude
2.840 input token
420 output token

Veri Gizliliği
Dosya cloud'a gönderilmedi.
12 anonim kayıt AI analizinde kullanıldı.
```

------------------------------------------------------------------------

# 17. Agent Çalışma Geçmişi

Kullanıcı geçmiş çalışmaları görebilir.

``` text
Bugün

09:42 Bank Reconciliation
✓ Tamamlandı

09:18 Supplier Comparison
✓ Tamamlandı

Dün

17:20 Expense Analysis
! Kullanıcı onayı bekliyor
```

Her execution tekrar açılabilir.

Gösterilecek bilgiler:

-   kullanılan dosyalar
-   yapılan işlemler
-   AI modeli
-   oluşturulan çıktılar
-   veri çıkış bilgisi
-   hata/uyarılar
-   workflow'a dönüştürme seçeneği

------------------------------------------------------------------------

# 18. Maliyet Görünürlüğü

BYOK kullanan kullanıcı kendi API maliyetini görebilmelidir.

Dashboard:

``` text
Bu Ay

AI Çağrısı       184
Input Token      1.24M
Output Token     186K
Tahmini Maliyet  $8.42

Cache ile Tasarruf
$3.18
```

Provider/model bazında filtrelenebilir.

------------------------------------------------------------------------

# 19. Cache Kullanıcı Deneyimi

Cache teknik bir özellik olmasına rağmen kullanıcıya faydası
gösterilebilir.

Örneğin:

``` text
Bu doküman daha önce işlendi.

✓ Parse cache kullanıldı
✓ Embedding tekrar oluşturulmadı

Tahmini süre kazancı: 14 sn
Tahmini AI tasarrufu: $0.04
```

Kullanıcı:

-   cache boyutunu görebilir,
-   cache temizleyebilir,
-   Knowledge Workspace indexini yeniden oluşturabilir.

------------------------------------------------------------------------

# 20. Company Workspace --- Paid

Şirket yöneticisi merkezi workspace oluşturabilir.

Workspace içinde:

``` text
Company Workspace

People
Departments
Agents
Workflows
Knowledge
Connections
Approvals
Policies
Audit
Usage
Settings
```

------------------------------------------------------------------------

# 21. Merkezi Kullanıcı ve Yetki Yönetimi

Paid sürümde:

-   kullanıcı
-   grup
-   departman
-   rol
-   agent erişimi
-   veri erişimi
-   tool yetkisi
-   action yetkisi

tanımlanabilir.

Örnek:

``` text
Finance Analyst
✓ Finance Knowledge
✓ Collection Agent
✓ CashFlow Agent
✓ ERP Receivable Read
✗ Payment Create
✗ Customer Credit Limit Change
```

------------------------------------------------------------------------

# 22. Company Knowledge

Paid sürüm şirketin ortak bilgi katmanını oluşturur.

Kaynaklar:

-   şirket dokümanları
-   prosedürler
-   politikalar
-   sözleşmeler
-   ERP
-   CRM
-   veritabanları
-   API'ler
-   paylaşılan klasörler

Agent'lar erişim yetkilerine göre aynı bilgi katmanını kullanabilir.

------------------------------------------------------------------------

# 23. Company Knowledge Graph

Şirket varlıkları arasındaki ilişkiler tutulabilir.

Örnek:

``` text
Customer ABC
   ↓
Contract
   ↓
Price Rule
   ↓
Sales Order
   ↓
Invoice
   ↓
Payment
```

veya:

``` text
Supplier ABC
   ↓
Contract
   ↓
Product X
   ↓
Purchase Orders
   ↓
Historical Price
```

Amaç yalnızca doküman bulmak değil, business context oluşturmaktır.

------------------------------------------------------------------------

# 24. Company Decision Memory

Paid sistem geçmiş kurumsal kararları yetki kontrollü şekilde
saklayabilir.

Agent daha sonra:

> Benzer durumlarda şirket daha önce ne yaptı?

sorusunu cevaplayabilir.

Decision Memory:

-   karar
-   bağlam
-   kullanılan kanıt
-   öneri
-   insan kararı
-   karar nedeni
-   onaylayan kişi/rol
-   uygulanan policy
-   tarih

içerebilir.

------------------------------------------------------------------------

# 25. ERP ve İş Sistemi Entegrasyonları

Paid ürün aşağıdaki sistem türlerine bağlanabilir:

-   ERP
-   CRM
-   muhasebe
-   veritabanı
-   REST API
-   SOAP API
-   dosya sistemleri
-   event/webhook sistemleri

ERP bağımsız canonical business model kullanılır.

Örnek entity'ler:

-   Customer
-   Supplier
-   Product
-   Inventory
-   SalesOrder
-   PurchaseOrder
-   Invoice
-   Payment
-   Shipment
-   Account

------------------------------------------------------------------------

# 26. Scheduled ve Event-Driven Agent

Free:

> Kullanıcı başlatır.

Paid:

> Sistem gerektiğinde kendisi çalıştırır.

Örnek:

``` text
Her iş günü 08:00
→ Tahsilat risklerini analiz et.

Yeni SalesOrder oluştu
→ Margin Guardian çalıştır.

Stok kritik seviyeye düştü
→ Procurement Agent çalıştır.

Yeni tedarikçi teklifi geldi
→ Supplier Comparison çalıştır.
```

------------------------------------------------------------------------

# 27. Approval Center

Agent kritik aksiyonları doğrudan yapmak zorunda değildir.

Örnek:

``` text
ONAY BEKLİYOR

Agent
Margin Guardian

Sipariş
SO-18429

Risk
Marj %4,2

Öneri
Siparişi bloke et.

Neden
Şirket minimum marj politikası %8.

[ Onayla ]
[ Reddet ]
[ Detay ]
```

Onay sonucu Decision Memory'ye aktarılabilir.

------------------------------------------------------------------------

# 28. Agent Yetki Seviyeleri

## Level 0 --- Read

Sadece okur.

## Level 1 --- Recommend

Analiz eder ve önerir.

## Level 2 --- Draft

İşlem taslağı hazırlar.

## Level 3 --- Approved Execute

İnsan onayından sonra işlemi gerçekleştirir.

## Level 4 --- Autonomous

Şirket policy'si içinde otomatik aksiyon alabilir.

Varsayılan olarak kritik finansal/operasyonel işlemler düşük yetkiyle
başlamalıdır.

------------------------------------------------------------------------

# 29. İlk Paid Agent --- Margin Guardian

Amaç:

> Sipariş gerçekleşmeden önce marj kaybını yakalamak.

Kontroller:

-   negatif marj
-   düşük marj
-   eski fiyat
-   maliyet artışı
-   olağandışı iskonto
-   müşteri özel fiyat farkı
-   tarihsel fiyat sapması

Örnek:

``` text
SO-29421

Sipariş Tutarı
245.000 TL

Mevcut Marj
%5,8

Beklenen Minimum
%11

Tahmini Marj Kaybı
12.740 TL

Risk Nedenleri
• %14 olağandışı iskonto
• maliyet geçen aya göre %8,2 arttı
• satış fiyatı güncellenmemiş

Öneri
Siparişi incelemeye al.
```

------------------------------------------------------------------------

# 30. Order Exception Agent

Normal siparişlerle ilgilenmez.

Sadece istisnaları yakalar:

-   kredi limiti
-   gecikmiş borç
-   düşük/negatif marj
-   stok problemi
-   olağandışı miktar
-   yanlış fiyat
-   aşırı iskonto
-   sıra dışı ödeme vadesi

Bu yaklaşım kullanıcıya sürekli gereksiz alarm üretmeyi azaltır.

------------------------------------------------------------------------

# 31. Paid Agent Çalışma Modeli

``` text
Observe
   ↓
Analyze
   ↓
Detect
   ↓
Recommend
   ↓
Policy
   ↓
Approval
   ↓
Execute
   ↓
Verify
   ↓
Audit
```

Agent'ın hangi noktaya kadar otomatik ilerleyebileceğini şirket
belirler.

------------------------------------------------------------------------

# 32. Merkezi Policy Yönetimi

Şirket agent davranışını kural olarak tanımlayabilir.

Örnek:

``` text
Margin < %5
→ sipariş mutlaka incelemeye alınır

Discount > %15
→ Sales Director approval

Purchase > 500.000 TL
→ Procurement Director + Finance approval

Supplier price increase > %10
→ ikinci teklif zorunlu
```

Agent bu kuralları LLM yorumuna bırakmadan policy engine üzerinden
uygular.

------------------------------------------------------------------------

# 33. Audit

Şirket aşağıdaki soruların cevabını görebilmelidir:

-   Hangi agent çalıştı?
-   Neden çalıştı?
-   Hangi verilere erişti?
-   Hangi modeli kullandı?
-   Hangi dokümanları kullandı?
-   Ne önerdi?
-   Hangi tool'u çağırdı?
-   Kim onayladı?
-   Hangi aksiyonu aldı?
-   Sonuç ne oldu?
-   Ne kadar AI maliyeti oluştu?

------------------------------------------------------------------------

# 34. Yönetim Dashboard'u

Paid dashboard örneği:

``` text
AI Workforce — Bu Ay

Agent Executions       18.420
Automated Workflows     3.240
Human Approvals           482

Risk Detected             317
Prevented Margin Loss   1.84M TL

AI Cost                  $428
Cache Saving             $173

Top Agents
1. Margin Guardian
2. Collection Agent
3. Procurement Agent
```

------------------------------------------------------------------------

# 35. Free → Paid Dönüşüm Mekanizması

Free kullanıcı ürün içinde doğal olarak Company Server ihtiyacını
görmelidir.

Örnek:

Kullanıcı workflow oluşturur.

> "Bunu her sabah otomatik çalıştır."

Ürün:

``` text
Bu workflow şu anda bilgisayarınızda manuel çalışıyor.

Company Workspace ile:
✓ Her sabah otomatik çalıştır
✓ Bilgisayar kapalıyken çalıştır
✓ Ekibinle paylaş
✓ ERP verisini otomatik al
✓ Sonucu yöneticine gönder
```

Başka örnek:

> "SAP'tan siparişleri otomatik çek."

→ Company Server özelliği.

Bu yaklaşım Free/Paid sınırını kullanıcıya doğal şekilde açıklar.

------------------------------------------------------------------------

# 36. Ürün Paketleme

## Free Desktop

Hedef:

-   bireysel çalışan
-   analyst
-   finance/accounting staff
-   procurement staff
-   operations staff
-   manager

Değer:

> Dosyalarla yaptığın günlük işleri AI worker'a yaptır.

## Company Server

Hedef:

-   KOBİ
-   orta/büyük şirket
-   operasyon ekipleri
-   finans
-   satış
-   satın alma
-   ERP kullanan şirketler

Değer:

> Şirket operasyonlarını AI agent'larla izle, analiz et ve kontrollü
> şekilde otomatikleştir.

------------------------------------------------------------------------

# 37. Ürün MVP Önceliği

## Free MVP

Öncelik:

1.  Desktop deneyimi
2.  BYOK
3.  Excel/CSV
4.  PDF/DOCX
5.  Local RAG
6.  Data Egress Guard
7.  İlk 9 agent
8.  Workflow save/replay
9.  Evidence-first analysis
10. Cache ve maliyet görünürlüğü

## Paid MVP

Öncelik:

1.  Company Workspace
2.  Tenant/User/RBAC
3.  Company Knowledge
4.  Connector altyapısı
5.  İlk ERP connector
6.  Policy
7.  Approval
8.  Scheduler/Event
9.  Margin Guardian
10. Order Exception Agent
11. Audit
12. Usage dashboard

------------------------------------------------------------------------

# 38. MVP'de Olmayacak Özellikler

İlk sürümde kapsam dışında:

-   genel amaçlı browser automation
-   sınırsız masaüstü kontrolü
-   her ERP'yi aynı anda destekleme
-   üçüncü taraf agent marketplace
-   kontrolsüz autonomous ERP write
-   LLM'in doğrudan SQL çalıştırması
-   kullanıcı izni olmadan full document cloud upload
-   yüzlerce yarım çalışan agent
-   genel amaçlı RPA ürünü olmaya çalışma

------------------------------------------------------------------------

# 39. Ürün Başarı Metrikleri

## Free

-   Weekly Active Users
-   Agent executions / user
-   Workflow replay rate
-   Successful task completion rate
-   RAG grounded answer rate
-   Time saved
-   Cache hit rate
-   Cloud data minimization rate
-   Free → Company interest/conversion

## Paid

-   Active company agents
-   Automated executions
-   Approval rate
-   Agent recommendation acceptance rate
-   False positive rate
-   Prevented loss / generated value
-   Manual work reduced
-   Workflow success rate
-   ERP action success rate
-   AI cost / business value

------------------------------------------------------------------------

# 40. Nihai Ürün Tanımı

AI Workforce üç katmanın birleşimidir:

``` text
Desktop Agent
Kullanıcının yaptığı işi bilir.
        +
Knowledge / RAG
Şirketin bilgisini bilir.
        +
Company Agent
Şirket sistemlerinde kontrollü aksiyon alabilir.
```

Uzun vadeli ürün:

> **Şirketlerin üzerinde çalışan bir AI Operating Layer.**

Ancak ürünün ilk sürümündeki vaat daha basit tutulmalıdır:

> **Günlük işlerini yapan departman bazlı AI çalışanları.**

Free kullanıcı için ürün somut bir işi çözmelidir.

Paid müşteri için ürün somut bir operasyonel veya finansal sonuç
üretmelidir.

Ürünün temel farkı chatbot olmak değil:

> **Bilgiyi okumak → işi anlamak → analiz etmek → karar önermek → yetki
> varsa aksiyon almak.**

# 41. Free Hesap ve Merkezi API Özellikleri

Free Desktop tamamen cloud'a bağımlı bir uygulama olmayacaktır.

Merkezi Go API yalnızca hesap ve ürün operasyonları için
kullanılacaktır.

## Merkezi API ile yapılacak işlemler

-   Login
-   Register
-   Logout
-   Session yenileme
-   Şifre sıfırlama
-   E-posta doğrulama
-   Kullanıcı profili
-   Ajan önerme
-   Hata gönderme
-   Genel geri bildirim
-   Uygulama sürüm kontrolü
-   Feature flag / güvenli remote config

Free agent'ın asıl işi olan:

-   dosya okuma,
-   Excel işleme,
-   doküman analizi,
-   local RAG,
-   workflow,
-   LLM orchestration,
-   cache,
-   Data Egress Guard

Rust tarafında lokal çalışmaya devam eder.

Merkezi API agent execution proxy'si değildir.

------------------------------------------------------------------------

# 42. Login / Register

İlk açılış akışı:

``` text
AI Workforce

[ Giriş Yap ]
[ Ücretsiz Hesap Oluştur ]

-----------------

E-posta
Şifre

[ Giriş Yap ]

Şifremi Unuttum
```

Register:

``` text
Ad
E-posta
Şifre
Şifre Tekrar

☑ Kullanım koşullarını kabul ediyorum
☑ Gizlilik politikasını okudum

[ Hesap Oluştur ]
```

Hesap açıldıktan sonra onboarding devam eder:

``` text
Hangi departmanda çalışıyorsun?
```

Bu bilgi başlangıç agent deneyimini kişiselleştirmek için
kullanılabilir.

------------------------------------------------------------------------

# 43. Ajan Öner

Free Desktop menüsünde:

**"Ajan Öner"**

özelliği bulunacaktır.

Kullanıcı:

-   departmanı,
-   çözmek istediği problemi,
-   beklediği sonucu,
-   örnek iş akışını

yazabilir.

Örnek:

``` text
Departman
Satın Alma

Önerdiğin ajan
Sözleşme Yenileme Risk Ajanı

Problem
Tedarikçi sözleşmelerindeki bitiş tarihlerini sürekli manuel kontrol ediyoruz.

Beklenen sonuç
Yaklaşan yenilemeleri ve riskli maddeleri göstersin.
```

Varsayılan olarak gerçek şirket dosyaları merkezi sisteme yüklenmez.

Bu öneriler admin panelinde ürün backlog'u oluşturmak için kullanılır.

------------------------------------------------------------------------

# 44. Hata Bildir

Kullanıcı agent veya uygulama hatasında:

**"Hata Bildir"**

seçeneğini kullanabilir.

Gönderilmeden önce kullanıcıya içerik gösterilir.

Örnek:

``` text
Hata Raporu

Uygulama Sürümü
1.2.4

İşletim Sistemi
macOS ARM64

Bileşen
Spreadsheet Engine

Hata Kodu
XLSX_PARSE_004

Açıklama
[ Kullanıcının açıklaması ]

Gönderilecek teknik bilgileri göster >

[ Hata Raporunu Gönder ]
```

Otomatik gönderilmemesi gerekenler:

-   API key
-   şirket dokümanları
-   Excel satırları
-   RAG kaynak içerikleri
-   hassas prompt
-   kullanıcı şifreleri
-   tam lokal dosya yolları

------------------------------------------------------------------------

# 45. Feedback

Ayrı bir feedback alanı bulunabilir.

Tipler:

-   Hata
-   Özellik isteği
-   Ajan kalitesi
-   Kullanım deneyimi
-   Performans
-   Gizlilik
-   Diğer

Bu kanal hata raporundan ayrıdır.

------------------------------------------------------------------------

# 46. Merkezi Backend Teknolojisi

Free ürünün merkezi control-plane backend'i:

``` text
Go API
+
PostgreSQL
```

olacaktır.

Go API:

-   authentication,
-   account,
-   session,
-   agent suggestions,
-   error reports,
-   feedback,
-   release metadata,
-   feature flags,
-   admin

işlerinden sorumludur.

Rust:

> Agent tarafındaki tüm local backend/runtime işlerinden sorumludur.

------------------------------------------------------------------------

# 47. Offline / API Kesintisi Davranışı

Merkezi API geçici olarak erişilemiyorsa:

Çalışmaya devam etmesi gerekenler:

-   local Excel işleri
-   local doküman işleri
-   local RAG
-   local workflow
-   Ollama/local model
-   mevcut local execution history

Etkilenebilecekler:

-   yeni login
-   yeni register
-   agent suggestion
-   error report submission
-   remote config
-   account yönetimi

Ürün merkezi API problemi nedeniyle kullanıcının lokal işini gereksiz
yere durdurmamalıdır.

------------------------------------------------------------------------

# 48. Admin Panel

Admin panel sadece ürün ekibinin kullanacağı operasyon ekranıdır.

Ana menü:

``` text
Dashboard
Kullanıcılar
Oturumlar / Cihazlar
Ajan Önerileri
Hata Raporları
Feedback
Agent Catalog
Feature Flags
Remote Config
Sürümler
Ürün Analitiği
Güvenlik
Audit
```

------------------------------------------------------------------------

# 49. Admin Dashboard

Gösterilebilecek metrikler:

-   toplam kullanıcı
-   günlük aktif kullanıcı
-   haftalık aktif kullanıcı
-   yeni kayıt
-   doğrulanmış hesap
-   kullanılan app sürümleri
-   işletim sistemi dağılımı
-   en çok kullanılan agent türleri
-   departman dağılımı
-   hata oranı
-   açık hata raporları
-   bekleyen agent önerileri
-   workflow kullanım oranı
-   RAG kullanım oranı
-   Free → Paid ilgi sinyalleri

Bu analitik için şirket dosyalarının içeriği merkezi sisteme alınmaz.

------------------------------------------------------------------------

# 50. Admin Kullanıcı Yönetimi

Admin:

-   kullanıcı arayabilir,
-   hesap durumunu görebilir,
-   e-posta doğrulama durumunu görebilir,
-   uygulama sürümünü görebilir,
-   bağlı cihaz/oturumları görebilir,
-   oturumları iptal edebilir,
-   hesabı devre dışı bırakabilir,
-   kullanıcı feedback'lerini görebilir,
-   kullanıcının agent önerilerini görebilir,
-   hata raporlarını görebilir.

Admin hiçbir zaman kullanıcının:

-   Claude/OpenAI/Gemini API key'ini,
-   lokal business dokümanlarını,
-   lokal RAG içeriğini

göremez.

------------------------------------------------------------------------

# 51. Agent Öneri Yönetimi

Admin ekranındaki kolonlar:

``` text
Durum
Departman
Başlık
Kullanıcı Sayısı / Benzer İstek
Öncelik
Oluşturulma Tarihi
Roadmap
```

Durum:

``` text
Yeni
İnceleniyor
Planlandı
Geliştiriliyor
Yayınlandı
Reddedildi
```

Benzer agent istekleri gruplanabilir.

Bu bölüm product discovery için ana kaynaklardan biri olacaktır.

------------------------------------------------------------------------

# 52. Hata Merkezi

Admin hata raporlarını fingerprint bazında gruplayabilir.

Örnek:

``` text
XLSX_PARSE_004

Severity
High

Affected Users
142

Occurrences
1.842

First Seen
...

Last Seen
...

Versions
1.2.3
1.2.4

OS
macOS ARM64 — %68
Windows x64 — %32
```

Admin:

-   severity belirleyebilir,
-   kişiye atayabilir,
-   internal note ekleyebilir,
-   resolved yapabilir,
-   hangi sürümde düzeldiğini belirtebilir.

------------------------------------------------------------------------

# 53. Agent Catalog Yönetimi

Admin product metadata yönetebilir:

-   agent adı
-   açıklama
-   departman
-   ikon
-   Free/Paid
-   aktif/pasif
-   beta
-   minimum uygulama sürümü
-   feature flag
-   release note

MVP'de admin panelinden doğrudan kontrolsüz executable agent kodu
yüklenmez.

Agent/skill paketleri imzalı ve versiyonlu release sürecinden geçer.

------------------------------------------------------------------------

# 54. Feature Flags

Admin bazı özellikleri kontrollü açabilir.

Örnek:

``` text
rag_v2
smart_model_router
procurement_decision_beta
new_onboarding
gemini_adapter_beta
```

Kullanım:

-   yüzde bazlı rollout
-   app version filtresi
-   platform filtresi
-   beta kullanıcıları

Feature flag güvenlik sınırı değildir.

Paid özelliğin lisans kontrolünü bypass edemez.

------------------------------------------------------------------------

# 55. Remote Config

Admin güvenli konfigürasyonları merkezi değiştirebilir.

Örnek:

-   minimum uygulama sürümü
-   önerilen sürüm
-   destek adresi
-   bakım mesajı
-   maksimum hata attachment boyutu
-   feature announcement
-   provider compatibility metadata

Remote Config:

-   Data Egress Guard'ı kapatamaz,
-   secret gönderemez,
-   dosya erişim yetkisi veremez,
-   Paid entitlement bypass edemez.

------------------------------------------------------------------------

# 56. Sürüm Yönetimi

Admin panelden:

-   stable
-   beta

kanalları yönetilebilir.

Sürüm için:

-   versiyon
-   platform
-   architecture
-   release note
-   minimum desteklenen versiyon
-   zorunlu güvenlik güncellemesi
-   staged rollout
-   artifact hash/signature metadata

tutulabilir.

------------------------------------------------------------------------

# 57. Admin Güvenlik ve Audit

Admin işlemleri role göre yetkilendirilir.

Örnek roller:

-   Super Admin
-   Product Admin
-   Support Admin
-   Security Admin
-   Read Only

Kritik işlemler audit edilir:

-   hesap disable
-   session revoke
-   feature flag değiştirme
-   release yayınlama
-   admin rol değiştirme

Audit kaydı:

``` text
Kim?
Ne yaptı?
Hangi kayıt üzerinde?
Ne zaman?
Hangi request?
Önceki durum?
Yeni durum?
```

------------------------------------------------------------------------

# 58. Free Ürün Mimari Özeti

``` text
Desktop UI
    ↓
Rust Local Backend
    ├── Agent Runtime
    ├── RAG
    ├── Tools
    ├── Cache
    ├── SQLite
    ├── BYOK LLM Gateway
    └── Data Egress Guard

Rust Desktop
    │
    │ sadece hesap/ürün operasyonları
    ▼
Go API
    ↓
PostgreSQL

Go API
    ├── Login/Register
    ├── Account
    ├── Session
    ├── Agent Suggest
    ├── Error Report
    ├── Feedback
    ├── Release
    ├── Feature Flags
    └── Admin
```

Bu ayrım ürünün gizlilik ve ölçekleme stratejisinin temelidir.
